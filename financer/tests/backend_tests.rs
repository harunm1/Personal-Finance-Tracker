// Unit tests for db.rs backend functions

#[cfg(test)]
mod tests {
    use diesel::sqlite::SqliteConnection;
    use diesel::Connection;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use diesel::prelude::*;
    use diesel::ExpressionMethods;
    use financer::db::*;
    use financer::models::*;
    use financer::schema::contacts::dsl::*;
    use chrono::NaiveDate;
    use chrono::Duration;

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    fn get_test_connection() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        conn
    }

    #[test]
    fn test_establish_connection_in_memory() {
        let _conn = SqliteConnection::establish(":memory:").unwrap();
        assert!(true); // If we get here, connection is established
    }

    #[test]
    fn test_create_user_and_get_userid_by_username() {
        let mut conn = get_test_connection();
        let username = "testuser";
        let password = "testpass";
        let email = Some("test@example.com");
        let res = create_user(&mut conn, username, password, email);
        assert!(res.is_ok());
        let user_obj = get_userid_by_username(&mut conn, username).unwrap();
        assert_eq!(user_obj.username, username);
    }

    #[test]
    fn test_create_account_and_get_user_accounts() {
        let mut conn = get_test_connection();
        // Insert user first
        let username = "accuser";
        create_user(&mut conn, username, "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, username).unwrap();
        // Create account
        let res = create_account(&mut conn, "Checking", "bank", 100.0, user_obj.id);
        assert!(res.is_ok());
        let accounts = get_user_accounts(&mut conn, user_obj.id).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Checking");
    }

    #[test]
    fn test_create_contact() {
        let mut conn = get_test_connection();
        create_user(&mut conn, "contactuser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "contactuser").unwrap();
        let res = create_contact(&mut conn, "Alice", user_obj.id);
        assert!(res.is_ok());
        let contact: Option<(i32, String, i32)> = contacts
            .filter(name.eq("Alice"))
            .filter(user.eq(user_obj.id))
            .select((id, name, user))
            .first::<(i32, String, i32)>(&mut conn)
            .optional()
            .unwrap();
        assert!(contact.is_some());
    }

    #[test]
    fn test_create_transaction_and_get_user_transactions() {
        let mut conn = get_test_connection();
        create_user(&mut conn, "txuser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "txuser").unwrap();
        create_account(&mut conn, "Main", "bank", 100.0, user_obj.id).unwrap();
        let accounts = get_user_accounts(&mut conn, user_obj.id).unwrap();
        let account = &accounts[0];
        create_contact(&mut conn, "Bob", user_obj.id).unwrap();
        let contact_id: i32 = contacts
            .filter(name.eq("Bob"))
            .filter(user.eq(user_obj.id))
            .select(id)
            .first(&mut conn)
            .unwrap();
        let res = create_transaction(&mut conn, account.id, contact_id, 50.0, "Food".to_string(), "2025-12-13".to_string());
        assert!(res.is_ok());
        let txs = get_user_transactions(&mut conn, user_obj.id).unwrap();
        assert!(!txs.is_empty());
    }

    #[test]
    fn test_create_transfer() {
        let mut conn = get_test_connection();
        create_user(&mut conn, "transuser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "transuser").unwrap();
        create_account(&mut conn, "A", "bank", 100.0, user_obj.id).unwrap();
        create_account(&mut conn, "B", "bank", 50.0, user_obj.id).unwrap();
        let accounts = get_user_accounts(&mut conn, user_obj.id).unwrap();
        let res = create_transfer(&mut conn, accounts[0].id, accounts[1].id, 25.0, "2025-12-13".to_string());
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_user() {
        let mut conn = get_test_connection();
        create_user(&mut conn, "verifyuser", "pass", None).unwrap();
        let ok = verify_user(&mut conn, "verifyuser", "pass").unwrap();
        assert!(ok);
        let fail = verify_user(&mut conn, "verifyuser", "wrong").unwrap();
        assert!(!fail);
    }

    #[test]
    fn test_create_and_update_budget() {
        let mut conn = get_test_connection();
        create_user(&mut conn, "budgetuser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "budgetuser").unwrap();
        let new_budget = NewBudget {
            user_id: user_obj.id,
            category: "Groceries".to_string(),
            limit_cents: 20000, // $200.00
            period: "monthly".to_string(),
            target_type: "spending".to_string(),
        };
        let budget = create_budget(&mut conn, new_budget).unwrap();
        assert_eq!(budget.category, "Groceries");
        let updated = update_budget(
            &mut conn,
            budget.id.expect("budget should have id"),
            NewBudget {
                user_id: user_obj.id,
                category: "Groceries".to_string(),
                limit_cents: 25000, // $250.00
                period: "monthly".to_string(),
                target_type: "spending".to_string(),
            },
        ).unwrap();
        assert_eq!(updated.limit_cents, 25000);
    }

    #[test]
    fn test_delete_budget() {
        let mut conn = get_test_connection();
        create_user(&mut conn, "delbudgetuser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "delbudgetuser").unwrap();
        let new_budget = NewBudget {
            user_id: user_obj.id,
            category: "Travel".to_string(),
            limit_cents: 10000, // $100.00
            period: "monthly".to_string(),
            target_type: "spending".to_string(),
        };
        let budget = create_budget(&mut conn, new_budget).unwrap();
        let res = delete_budget(&mut conn, budget.id.expect("budget should have id"));
        assert!(res.is_ok());
    }

    #[test]
    fn test_get_spend_for_category_period_and_by_category_period() {
        let mut conn = get_test_connection();
        // Setup user, account, and transaction
        create_user(&mut conn, "spenduser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "spenduser").unwrap();
        create_account(&mut conn, "Main", "bank", 1000.0, user_obj.id).unwrap();
        let accounts = get_user_accounts(&mut conn, user_obj.id).unwrap();
        let account = &accounts[0];
        create_contact(&mut conn, "Store", user_obj.id).unwrap();
        let contact_id: i32 = contacts
            .filter(name.eq("Store"))
            .filter(user.eq(user_obj.id))
            .select(id)
            .first(&mut conn)
            .unwrap();
        // Add transactions in two categories
        create_transaction(&mut conn, account.id, contact_id, 80.0, "Groceries".to_string(), "2025-12-01 00:00:00".to_string()).unwrap();
        create_transaction(&mut conn, account.id, contact_id, 20.0, "Transport".to_string(), "2025-12-03 00:00:00".to_string()).unwrap();
        let start_date = NaiveDate::parse_from_str("2025-12-01", "%Y-%m-%d").unwrap().and_hms_opt(0,0,0).unwrap();
        let end_date = NaiveDate::parse_from_str("2025-12-31", "%Y-%m-%d").unwrap().and_hms_opt(23,59,59).unwrap();
        let end_date_inclusive = end_date + Duration::days(1);
        let spend = get_spend_for_category_period(&mut conn, user_obj.id, "Groceries", start_date, end_date_inclusive).unwrap();
        assert_eq!(spend, 8000); // 80.00 dollars = 8000 cents
        let spend_transport = get_spend_for_category_period(&mut conn, user_obj.id, "Transport", start_date, end_date_inclusive).unwrap();
        assert_eq!(spend_transport, 2000); // 20.00 dollars = 2000 cents
        let spend_by_cat = get_spend_by_category_period(&mut conn, user_obj.id, start_date, end_date_inclusive).unwrap();
        assert!(spend_by_cat.iter().any(|(cat, amt)| cat == "Groceries" && *amt == 8000));
        assert!(spend_by_cat.iter().any(|(cat, amt)| cat == "Transport" && *amt == 2000));
    }

    #[test]
    fn test_get_user_categories() {
        let mut conn = get_test_connection();
        // Setup user, account, transaction, and budget
        create_user(&mut conn, "catuser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "catuser").unwrap();
        create_account(&mut conn, "Main", "bank", 100.0, user_obj.id).unwrap();
        let accounts = get_user_accounts(&mut conn, user_obj.id).unwrap();
        let account = &accounts[0];
        create_contact(&mut conn, "Vendor", user_obj.id).unwrap();
        let contact_id: i32 = contacts
            .filter(name.eq("Vendor"))
            .filter(user.eq(user_obj.id))
            .select(id)
            .first(&mut conn)
            .unwrap();
        // Add transactions in two categories
        create_transaction(&mut conn, account.id, contact_id, 1000.0, "Books".to_string(), "2025-12-01".to_string()).unwrap();
        create_transaction(&mut conn, account.id, contact_id, 2000.0, "Music".to_string(), "2025-12-02".to_string()).unwrap();
        // Add a budget in a third category
        let new_budget = NewBudget {
            user_id: user_obj.id,
            category: "Travel".to_string(),
            limit_cents: 10000,
            period: "monthly".to_string(),
            target_type: "spending".to_string(),
        };
        create_budget(&mut conn, new_budget).unwrap();
        // Now get categories
        let result = get_user_categories(&mut conn, user_obj.id).unwrap();
        // Should contain Books, Music, and Travel
        assert!(result.contains(&"Books".to_string()));
        assert!(result.contains(&"Music".to_string()));
        assert!(result.contains(&"Travel".to_string()));
    }

    #[test]
    fn test_update_and_delete_transaction() {
        let mut conn = get_test_connection();
        // Setup user, account, transaction
        create_user(&mut conn, "updeluser", "pass", None).unwrap();
        let user_obj = get_userid_by_username(&mut conn, "updeluser").unwrap();
        create_account(&mut conn, "Main", "bank", 100.0, user_obj.id).unwrap();
        let accounts = get_user_accounts(&mut conn, user_obj.id).unwrap();
        let account = &accounts[0];
        create_contact(&mut conn, "Shop", user_obj.id).unwrap();
        let contact_id: i32 = contacts
            .filter(name.eq("Shop"))
            .filter(user.eq(user_obj.id))
            .select(id)
            .first(&mut conn)
            .unwrap();
        // Create a transaction and get its id by fetching the latest transaction
        create_transaction(&mut conn, account.id, contact_id, 15.0, "Snacks".to_string(), "2025-12-10".to_string()).unwrap();
        let txs = get_user_transactions(&mut conn, user_obj.id).unwrap();
        let tx = txs.last().expect("should have a transaction");
        let tx_id = tx.id;
        // Update the transaction: (conn, tx_id, user_account_id, amount, category, date)
        let updated_rows = update_transaction(
            &mut conn,
            tx_id,
            account.id,
            25.0,
            "Dining".to_string(),
            "2025-12-11".to_string(),
        ).unwrap();
        assert_eq!(updated_rows, 1); // Should update one row
        // Fetch the updated transaction
        let txs = get_user_transactions(&mut conn, user_obj.id).unwrap();
        let updated_tx = txs.iter().find(|t| t.id == tx_id).unwrap();
        assert!((updated_tx.amount - 25.0).abs() < 1e-6);
        assert_eq!(updated_tx.category, "Dining");
        assert_eq!(updated_tx.date, "2025-12-11");
        // Delete the transaction
        let res = delete_transaction(&mut conn, tx_id);
        assert!(res.is_ok());
        // Should not find the transaction anymore
        let txs = get_user_transactions(&mut conn, user_obj.id).unwrap();
        assert!(!txs.iter().any(|t| t.id == tx_id));
    }
}