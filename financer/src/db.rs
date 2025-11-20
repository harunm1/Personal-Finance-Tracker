use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::Error;
use crate::models::{User, NewUser, NewAccount, NewContact, NewTransaction, AccountType, Account};
use crate::schema::users::dsl::*;
use crate::schema::accounts::dsl::*;
use crate::schema::contacts::dsl::*;
use crate::schema::transactions::dsl::*;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use ::password_hash::{SaltString, PasswordHash};
use email_address::EmailAddress;

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

pub fn create_transaction(conn: &mut SqliteConnection, new_user_account: i32, new_contact_id: i32, new_amount: f32) -> Result<usize, Error> {
    let new_transaction = NewTransaction {
        user_account_id: new_user_account,
        contact_id: new_contact_id,
        amount: new_amount,
    };
    
    diesel::insert_into(transactions).values(&new_transaction).execute(conn)
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

