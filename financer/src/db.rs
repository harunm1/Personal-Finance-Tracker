use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::Error;
use crate::models::{User, NewUser, NewAccount, NewContact, NewTransaction, Account, Transaction};
use crate::schema::users::dsl::*;
use crate::schema::accounts::dsl::*;
use crate::schema::contacts::dsl::*;
use crate::schema::transactions::dsl::*;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use ::password_hash::{SaltString, PasswordHash};
use email_address::EmailAddress;

use crate::models::{Budget, NewBudget};
use chrono::NaiveDateTime;
use diesel::dsl::sum;

pub fn establish_connection() -> SqliteConnection {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    SqliteConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url))
}

pub fn create_user(conn: &mut SqliteConnection, new_username: &str, new_password: &str, new_email: Option<&str>) -> Result<usize, Error> {
    // check if user already exists
    if users.filter(username.eq(new_username)).first::<User>(conn).optional()?.is_some() {
        println!("User already exists: {}", new_username);
        return Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, Box::new("Username already exists".to_string())));
    }
    // validate email if provided
    if let Some(email_str) = new_email {
        if !EmailAddress::is_valid(email_str) {
            println!("Invalid email format: {}", email_str);
            return Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, Box::new("Invalid email format".to_string())));
        }
    }
    // Hash the password
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
    accounts.filter(user_id.eq(owner_id)).load::<Account>(conn)
}

pub fn get_userid_by_username(conn: &mut SqliteConnection, search_username: &str) -> Result<User, Error> {
    users.filter(username.eq(search_username)).first::<User>(conn)
}

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
    
    // Get current account balance
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
    
    // Update account balance
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
    // Use a transaction to ensure atomicity
    conn.transaction::<_, Error, _>(|conn| {
        // Create withdrawal from source account
        create_transaction(
            conn,
            from_account_id,
            0, // no contact
            -transfer_amount.abs(), // negative for withdrawal
            "Transfer".to_string(),
            transfer_date.clone(),
        )?;
        
        // Create deposit to destination account
        create_transaction(
            conn,
            to_account_id,
            0, // no contact
            transfer_amount.abs(), // positive for deposit
            "Transfer".to_string(),
            transfer_date,
        )?;
        
        Ok(())
    })
}

pub fn verify_user(conn: &mut SqliteConnection, login_username: &str, login_password: &str) -> Result<bool, Error> {
    // Try to find username
    match users.filter(username.eq(login_username)).first::<User>(conn){
        Ok(u) => {
            // Verify the hashed password
            let parsed_hash = PasswordHash::new(&u.password_hash).expect("Error parsing password hash");
            let argon2 = Argon2::default();
            Ok(argon2.verify_password(login_password.as_bytes(), &parsed_hash).is_ok())
        }
        // User not found
        Err(diesel::result::Error::NotFound) => Ok(false),
        // Any other errors 
        Err(e) => Err(e),
    }
}




//Budgeting functions

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

// Aggregation: total spent/received in a period for a category across all user accounts.
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

// Get all transactions for a user (across all their accounts)
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

// Get all unique categories used by a user (from both budgets and transactions)
pub fn get_user_categories(conn: &mut SqliteConnection, owner_id: i32) -> Result<Vec<String>, Error> {
    use crate::schema::transactions::dsl::*;
    use crate::schema::accounts;
    use crate::schema::budgets;
    
    // Get categories from transactions
    let tx_categories: Vec<String> = transactions
        .inner_join(accounts::table.on(user_account_id.eq(accounts::id)))
        .filter(accounts::user_id.eq(owner_id))
        .select(category)
        .distinct()
        .load::<String>(conn)?;
    
    // Get categories from budgets
    let budget_categories: Vec<String> = budgets::table
        .filter(budgets::user_id.eq(owner_id))
        .filter(budgets::active.eq(true))
        .select(budgets::category)
        .distinct()
        .load::<String>(conn)?;
    
    // Merge and deduplicate
    let mut all_categories: Vec<String> = tx_categories.into_iter()
        .chain(budget_categories.into_iter())
        .collect();
    all_categories.sort();
    all_categories.dedup();
    
    Ok(all_categories)
}

// Update a transaction
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
    
    // Save old transaction to revert its balance impact
    let old_tx: Transaction = transactions.filter(id.eq(transaction_id)).first(conn)?;
    
    let cents = (new_amount * 100.0) as i32;
    
    // Revert old transaction's balance impact
    update_account_balance(conn, old_tx.user_account_id, -old_tx.amount)?;
    
    // Apply new transaction's balance impact
    update_account_balance(conn, new_user_account, new_amount)?;
    
    // Get the new balance after update
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

// Delete a transaction
pub fn delete_transaction(conn: &mut SqliteConnection, transaction_id: i32) -> Result<usize, Error> {
    use crate::schema::transactions::dsl::*;
    
    // Save old transaction to revert its balance impact
    let old_tx: Transaction = transactions.filter(id.eq(transaction_id)).first(conn)?;
    
    let result = diesel::delete(transactions.filter(id.eq(transaction_id)))
        .execute(conn)?;
    
    // Revert the transaction's balance impact
    update_account_balance(conn, old_tx.user_account_id, -old_tx.amount)?;
    
    Ok(result)
}

// Helper function to update account balance
fn update_account_balance(
    conn: &mut SqliteConnection,
    account_id: i32,
    amount_change: f32,
) -> Result<usize, Error> {
    use crate::schema::accounts::dsl::*;
    
    // Get current balance
    let current_account: Account = accounts.filter(id.eq(account_id)).first(conn)?;
    let new_balance = current_account.balance + amount_change;
    
    diesel::update(accounts.filter(id.eq(account_id)))
        .set(balance.eq(new_balance))
        .execute(conn)
}