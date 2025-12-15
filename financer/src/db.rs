use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::Error;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use crate::models::{
    User,
    NewUser,
    NewAccount,
    NewContact,
    NewTransaction,
    Account,
    Transaction,
    RecurringTransaction,
    NewRecurringTransaction,
    RecurringTransfer,
    NewRecurringTransfer,
};
use crate::schema::users::dsl::*;
use crate::schema::accounts::dsl::*;
use crate::schema::contacts::dsl::*;
use crate::schema::transactions::dsl::*;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use ::password_hash::{SaltString, PasswordHash};
use email_address::EmailAddress;

use crate::models::{Budget, NewBudget};
use crate::models::Period;
use chrono::NaiveDateTime;
use diesel::dsl::sum;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection() -> SqliteConnection {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let mut conn = SqliteConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url));
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
    conn
}

pub fn create_user(conn: &mut SqliteConnection, new_username: &str, new_password: &str, new_email: Option<&str>) -> Result<usize, Error> {
    if users.filter(username.eq(new_username)).first::<User>(conn).optional()?.is_some() {
        println!("User already exists: {}", new_username);
        return Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, Box::new("Username already exists".to_string())));
    }
    if let Some(email_str) = new_email {
        if !EmailAddress::is_valid(email_str) {
            println!("Invalid email format: {}", email_str);
            return Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, Box::new("Invalid email format".to_string())));
        }
    }
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(new_password.as_bytes(), &salt).expect("Error hashing password").to_string();

    let new_user = NewUser {
        username: new_username,
        password_hash: &hashed_password,
        email: new_email,
    };

    diesel::insert_into(users).values(&new_user).execute(conn)
}

pub fn create_account(conn: &mut SqliteConnection, new_name: &str, new_account_type: &str, new_balance: f32, account_owner_id: i32) -> Result<usize, Error> {
    let new_account = NewAccount {
        name: new_name,
        account_type: new_account_type,
        balance: new_balance,
        user_id: account_owner_id,
    };

    diesel::insert_into(accounts).values(&new_account).execute(conn)
}

pub fn get_user_accounts(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<Account>, Error> {
    accounts
        .filter(user_id.eq(owner_id))
        .filter(active.eq(true))
        .load::<Account>(conn)
}

pub fn delete_user_and_all_data(conn: &mut SqliteConnection, owner_id: i32) -> Result<(), Error> {
    conn.transaction::<_, Error, _>(|conn| {
        use crate::schema::{accounts, budgets, contacts, recurring_transactions, recurring_transfers, transactions, users};

        let account_ids: Vec<i32> = accounts::table
            .filter(accounts::user_id.eq(owner_id))
            .select(accounts::id)
            .load(conn)?;

        if !account_ids.is_empty() {
            diesel::delete(
                transactions::table.filter(transactions::user_account_id.eq_any(&account_ids)),
            )
            .execute(conn)?;
        }

        diesel::delete(recurring_transactions::table.filter(recurring_transactions::user_id.eq(owner_id)))
            .execute(conn)?;
        diesel::delete(recurring_transfers::table.filter(recurring_transfers::user_id.eq(owner_id)))
            .execute(conn)?;
        diesel::delete(budgets::table.filter(budgets::user_id.eq(owner_id))).execute(conn)?;
        diesel::delete(contacts::table.filter(contacts::user.eq(owner_id))).execute(conn)?;
        diesel::delete(accounts::table.filter(accounts::user_id.eq(owner_id))).execute(conn)?;
        diesel::delete(users::table.filter(users::id.eq(owner_id))).execute(conn)?;

        Ok(())
    })
}

pub fn delete_account(conn: &mut SqliteConnection, owner_id: i32, account_id: i32) -> Result<usize, Error> {
    diesel::update(
        accounts
            .filter(crate::schema::accounts::dsl::id.eq(account_id))
            .filter(crate::schema::accounts::dsl::user_id.eq(owner_id)),
    )
        .set(crate::schema::accounts::dsl::active.eq(false))
        .execute(conn)
}

fn add_months_clamped(dt: chrono::NaiveDateTime, months: i32) -> chrono::NaiveDateTime {
    use chrono::{Datelike, NaiveDate, Timelike};

    let start_year = dt.date().year();
    let start_month0 = dt.date().month0() as i32;
    let total_month0 = start_month0 + months;

    let mut year = start_year + total_month0.div_euclid(12);
    let mut month0 = total_month0.rem_euclid(12);
    if month0 < 0 {
        month0 += 12;
        year -= 1;
    }
    let month = (month0 + 1) as u32;

    let day = dt.date().day();
    let mut candidate = NaiveDate::from_ymd_opt(year, month, day);
    if candidate.is_none() {
        let mut d = day;
        while d > 28 {
            d -= 1;
            candidate = NaiveDate::from_ymd_opt(year, month, d);
            if candidate.is_some() {
                break;
            }
        }
    }

    let new_date = candidate.unwrap();
    new_date
        .and_hms_opt(dt.time().hour(), dt.time().minute(), dt.time().second())
        .unwrap()
}

fn add_period(dt: chrono::NaiveDateTime, period: Period) -> chrono::NaiveDateTime {
    use chrono::Duration;

    match period {
        Period::Daily => dt + Duration::days(1),
        Period::Weekly => dt + Duration::days(7),
        Period::Monthly => add_months_clamped(dt, 1),
        Period::Yearly => add_months_clamped(dt, 12),
    }
}

pub fn get_user_recurring_transactions(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<RecurringTransaction>, Error> {
    use crate::schema::recurring_transactions::dsl::*;

    recurring_transactions
        .filter(user_id.eq(owner_id))
        .filter(active.eq(true))
        .order(next_run_at.asc())
        .load::<RecurringTransaction>(conn)
}

pub fn create_recurring_transaction(conn: &mut SqliteConnection, new_item: NewRecurringTransaction) -> Result<RecurringTransaction, Error> {
    use crate::schema::recurring_transactions::dsl::*;

    diesel::insert_into(recurring_transactions)
        .values(&new_item)
        .execute(conn)?;
    recurring_transactions.order(id.desc()).first(conn)
}

pub fn update_recurring_transaction(conn: &mut SqliteConnection, owner_id: i32, item_id: i32, changes: NewRecurringTransaction) -> Result<usize, Error> {
    use crate::schema::recurring_transactions::dsl::*;

    diesel::update(recurring_transactions.filter(id.eq(item_id)).filter(user_id.eq(owner_id)))
        .set((
            account_id.eq(changes.account_id),
            contact_id.eq(changes.contact_id),
            amount.eq(changes.amount),
            category.eq(changes.category),
            next_run_at.eq(changes.next_run_at),
            frequency.eq(changes.frequency),
        ))
        .execute(conn)
}

pub fn delete_recurring_transaction(conn: &mut SqliteConnection, owner_id: i32, item_id: i32) -> Result<usize, Error> {
    use crate::schema::recurring_transactions::dsl::*;

    diesel::delete(recurring_transactions.filter(id.eq(item_id)).filter(user_id.eq(owner_id)))
        .execute(conn)
}

pub fn get_user_recurring_transfers(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<RecurringTransfer>, Error> {
    use crate::schema::recurring_transfers::dsl::*;

    recurring_transfers
        .filter(user_id.eq(owner_id))
        .filter(active.eq(true))
        .order(next_run_at.asc())
        .load::<RecurringTransfer>(conn)
}

pub fn create_recurring_transfer(conn: &mut SqliteConnection, new_item: NewRecurringTransfer) -> Result<RecurringTransfer, Error> {
    use crate::schema::recurring_transfers::dsl::*;

    diesel::insert_into(recurring_transfers)
        .values(&new_item)
        .execute(conn)?;
    recurring_transfers.order(id.desc()).first(conn)
}

pub fn update_recurring_transfer(conn: &mut SqliteConnection, owner_id: i32, item_id: i32, changes: NewRecurringTransfer) -> Result<usize, Error> {
    use crate::schema::recurring_transfers::dsl::*;

    diesel::update(recurring_transfers.filter(id.eq(item_id)).filter(user_id.eq(owner_id)))
        .set((
            from_account_id.eq(changes.from_account_id),
            to_account_id.eq(changes.to_account_id),
            amount.eq(changes.amount),
            next_run_at.eq(changes.next_run_at),
            frequency.eq(changes.frequency),
        ))
        .execute(conn)
}

pub fn delete_recurring_transfer(conn: &mut SqliteConnection, owner_id: i32, item_id: i32) -> Result<usize, Error> {
    use crate::schema::recurring_transfers::dsl::*;

    diesel::delete(recurring_transfers.filter(id.eq(item_id)).filter(user_id.eq(owner_id)))
        .execute(conn)
}

pub fn process_due_recurring(conn: &mut SqliteConnection, owner_id: i32, now: chrono::NaiveDateTime) -> Result<usize, Error> {
    conn.transaction::<_, Error, _>(|conn| {
        use crate::schema::{recurring_transactions, recurring_transfers};
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let due_txs: Vec<RecurringTransaction> = recurring_transactions::table
            .filter(recurring_transactions::user_id.eq(owner_id))
            .filter(recurring_transactions::active.eq(true))
            .filter(recurring_transactions::next_run_at.le(&now_str))
            .order(recurring_transactions::next_run_at.asc())
            .load(conn)?;

        let due_transfers: Vec<RecurringTransfer> = recurring_transfers::table
            .filter(recurring_transfers::user_id.eq(owner_id))
            .filter(recurring_transfers::active.eq(true))
            .filter(recurring_transfers::next_run_at.le(&now_str))
            .order(recurring_transfers::next_run_at.asc())
            .load(conn)?;

        let mut processed = 0usize;

        for item in due_txs {
            let mut dt = chrono::NaiveDateTime::parse_from_str(&item.next_run_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or(now);
            let period = Period::from_str(&item.frequency);
            let mut iterations = 0;

            while dt <= now && iterations < 100 {
                create_transaction(
                    conn,
                    item.account_id,
                    item.contact_id,
                    item.amount,
                    item.category.clone(),
                    dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                )?;
                processed += 1;
                dt = add_period(dt, period);
                iterations += 1;
            }

            diesel::update(recurring_transactions::table.filter(recurring_transactions::id.eq(item.id)))
                .set(recurring_transactions::next_run_at.eq(dt.format("%Y-%m-%d %H:%M:%S").to_string()))
                .execute(conn)?;
        }

        for item in due_transfers {
            let mut dt = chrono::NaiveDateTime::parse_from_str(&item.next_run_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or(now);
            let period = Period::from_str(&item.frequency);
            let mut iterations = 0;

            while dt <= now && iterations < 100 {
                create_transfer(
                    conn,
                    item.from_account_id,
                    item.to_account_id,
                    item.amount,
                    dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                )?;
                processed += 1;
                dt = add_period(dt, period);
                iterations += 1;
            }

            diesel::update(recurring_transfers::table.filter(recurring_transfers::id.eq(item.id)))
                .set(recurring_transfers::next_run_at.eq(dt.format("%Y-%m-%d %H:%M:%S").to_string()))
                .execute(conn)?;
        }

        Ok(processed)
    })
}

pub fn get_userid_by_username(conn: &mut SqliteConnection, search_username: &str) -> Result<User, Error> {
    users.filter(username.eq(search_username)).first::<User>(conn)
}

#[allow(dead_code)]
pub fn create_contact(conn: &mut SqliteConnection, new_name: &str, new_user: i32) -> Result<usize, Error> {
    let new_contact = NewContact {
        name: new_name,
        user: new_user,
    };

    diesel::insert_into(contacts).values(&new_contact).execute(conn)
}

pub fn create_transaction(
    conn: &mut SqliteConnection,
    new_user_account: i32,
    new_contact_id: i32,
    new_amount: f32,
    new_category: String,
    new_date: String,
) -> Result<usize, Error> {
    use crate::schema::accounts::dsl::*;
    
    let cents = (new_amount * 100.0) as i32;
    
    let current_account: Account = accounts.filter(id.eq(new_user_account)).first(conn)?;
    let new_balance = current_account.balance + new_amount;
    
    let new_transaction = NewTransaction {
        user_account_id: new_user_account,
        contact_id: new_contact_id,
        amount: new_amount,
        category: new_category,
        date: new_date,
        amount_cents: cents,
        balance_after: new_balance,
    };
    
    let result = diesel::insert_into(transactions).values(&new_transaction).execute(conn)?;
    
    update_account_balance(conn, new_user_account, new_amount)?;
    
    Ok(result)
}

pub fn create_transfer(
    conn: &mut SqliteConnection,
    from_account_id: i32,
    to_account_id: i32,
    transfer_amount: f32,
    transfer_date: String,
) -> Result<(), Error> {
    conn.transaction::<_, Error, _>(|conn| {
        create_transaction(
            conn,
            from_account_id,
            0,
            -transfer_amount.abs(),
            "Transfer".to_string(),
            transfer_date.clone(),
        )?;
        
        create_transaction(
            conn,
            to_account_id,
            0,
            transfer_amount.abs(),
            "Transfer".to_string(),
            transfer_date,
        )?;
        
        Ok(())
    })
}

pub fn verify_user(conn: &mut SqliteConnection, login_username: &str, login_password: &str) -> Result<bool, Error> {
    match users.filter(username.eq(login_username)).first::<User>(conn){
        Ok(u) => {
            let parsed_hash = PasswordHash::new(&u.password_hash).expect("Error parsing password hash");
            let argon2 = Argon2::default();
            Ok(argon2.verify_password(login_password.as_bytes(), &parsed_hash).is_ok())
        }
        Err(diesel::result::Error::NotFound) => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn create_budget(conn: &mut SqliteConnection, new_budget: NewBudget) -> Result<Budget, Error> {
    use crate::schema::budgets::dsl::*;
    
    diesel::insert_into(budgets)
        .values(&new_budget)
        .execute(conn)?;
    
    budgets.order(id.desc()).first(conn)
}

pub fn get_user_budgets(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<Budget>, Error> {
    use crate::schema::budgets::dsl::*;
    
    budgets
        .filter(user_id.eq(owner_id))
        .filter(active.eq(true))
        .load::<Budget>(conn)
}

pub fn update_budget(conn: &mut SqliteConnection, budget_id: i32, changes: NewBudget) -> Result<Budget, Error> {
    use crate::schema::budgets::dsl::*;
    
    diesel::update(budgets.filter(id.eq(budget_id)))
        .set((
            category.eq(&changes.category),
            limit_cents.eq(&changes.limit_cents),
            period.eq(&changes.period),
            target_type.eq(&changes.target_type),
        ))
        .execute(conn)?;
    
    budgets.filter(id.eq(budget_id)).first(conn)
}

pub fn delete_budget(conn: &mut SqliteConnection, budget_id: i32) -> Result<usize, Error> {
    use crate::schema::budgets::dsl::*;
    
    diesel::delete(budgets.filter(id.eq(budget_id)))
        .execute(conn)
}

pub fn get_spend_for_category_period(
    conn: &mut SqliteConnection,
    owner_id: i32,
    cat: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<i64, Error> {
    use crate::schema::transactions::dsl::*;
    use crate::schema::accounts;
    
    let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let result: Option<Option<i64>> = transactions
        .inner_join(accounts::table.on(user_account_id.eq(accounts::id)))
        .filter(accounts::user_id.eq(owner_id))
        .filter(category.eq(cat))
        .filter(date.ge(start_str))
        .filter(date.lt(end_str))
        .select(sum(amount_cents))
        .first(conn)
        .optional()?;
    
    Ok(result.flatten().unwrap_or(0))
}

#[allow(dead_code)]
pub fn get_spend_by_category_period(
    conn: &mut SqliteConnection,
    owner_id: i32,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<Vec<(String, i64)>, Error> {
    use crate::schema::transactions::dsl::*;
    use crate::schema::accounts;
    use diesel::dsl::sum;
    
    let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end.format("%Y-%m-%d %H:%M:%S").to_string();
    
    transactions
        .inner_join(accounts::table.on(user_account_id.eq(accounts::id)))
        .filter(accounts::user_id.eq(owner_id))
        .filter(date.ge(start_str))
        .filter(date.lt(end_str))
        .group_by(category)
        .select((category, sum(amount_cents)))
        .load::<(String, Option<i64>)>(conn)
        .map(|results| {
            results
                .into_iter()
                .map(|(cat, amt)| (cat, amt.unwrap_or(0)))
                .collect()
        })
}

pub fn get_user_transactions(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<Transaction>, Error> {
    use crate::schema::transactions::dsl::*;
    use crate::schema::accounts;
    
    transactions
        .inner_join(accounts::table.on(user_account_id.eq(accounts::id)))
        .filter(accounts::user_id.eq(owner_id))
        .order(date.desc())
        .select((id, user_account_id, contact_id, amount, category, date, amount_cents, balance_after))
        .load::<Transaction>(conn)
}

pub fn get_user_categories(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<String>, Error> {
    use crate::schema::transactions::dsl::*;
    use crate::schema::accounts;
    use crate::schema::budgets;
    
    let tx_categories: Vec<String> = transactions
        .inner_join(accounts::table.on(user_account_id.eq(accounts::id)))
        .filter(accounts::user_id.eq(owner_id))
        .select(category)
        .distinct()
        .load::<String>(conn)?;
    
    let budget_categories: Vec<String> = budgets::table
        .filter(budgets::user_id.eq(owner_id))
        .filter(budgets::active.eq(true))
        .select(budgets::category)
        .distinct()
        .load::<String>(conn)?;
    
    let mut all_categories: Vec<String> = tx_categories.into_iter()
        .chain(budget_categories.into_iter())
        .collect();
    all_categories.sort();
    all_categories.dedup();
    
    Ok(all_categories)
}

pub fn update_transaction(
    conn: &mut SqliteConnection,
    transaction_id: i32,
    new_user_account: i32,
    new_amount: f32,
    new_category: String,
    new_date: String,
) -> Result<usize, Error> {
    use crate::schema::transactions::dsl::*;
    use crate::schema::accounts;
    
    let old_tx: Transaction = transactions.filter(id.eq(transaction_id)).first(conn)?;
    
    let cents = (new_amount * 100.0) as i32;
    
    update_account_balance(conn, old_tx.user_account_id, -old_tx.amount)?;

    update_account_balance(conn, new_user_account, new_amount)?;
    
    let current_account: Account = accounts::table.filter(accounts::id.eq(new_user_account)).first(conn)?;
    let new_balance_after = current_account.balance;
    
    let result = diesel::update(transactions.filter(id.eq(transaction_id)))
        .set((
            user_account_id.eq(new_user_account),
            amount.eq(new_amount),
            category.eq(&new_category),
            date.eq(&new_date),
            amount_cents.eq(cents),
            balance_after.eq(new_balance_after),
        ))
        .execute(conn)?;
    
    Ok(result)
}

pub fn delete_transaction(conn: &mut SqliteConnection, transaction_id: i32) -> Result<usize, Error> {
    use crate::schema::transactions::dsl::*;
    
    let old_tx: Transaction = transactions.filter(id.eq(transaction_id)).first(conn)?;
    
    let result = diesel::delete(transactions.filter(id.eq(transaction_id)))
        .execute(conn)?;
    
    update_account_balance(conn, old_tx.user_account_id, -old_tx.amount)?;
    
    Ok(result)
}

fn update_account_balance(
    conn: &mut SqliteConnection,
    account_id: i32,
    amount_change: f32,
) -> Result<usize, Error> {
    use crate::schema::accounts::dsl::*;
    
    let current_account: Account = accounts.filter(id.eq(account_id)).first(conn)?;
    let new_balance = current_account.balance + amount_change;
    
    diesel::update(accounts.filter(id.eq(account_id)))
        .set(balance.eq(new_balance))
        .execute(conn)
}