use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::Error;
use crate::models::{User, NewUser};
use crate::schema::users::dsl::*;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use ::password_hash::{SaltString, PasswordHash};

pub fn establish_connection() -> SqliteConnection {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    SqliteConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url))
}

pub fn create_user(conn: &mut SqliteConnection, new_username: &str, new_password: &str, new_email: Option<&str>) -> Result<usize, Error> {
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

pub fn verify_user(conn: &mut SqliteConnection, login_username: &str, login_password: &str) -> Result<bool, Error> {
    // Try to find username
    match users.filter(username.eq(login_username)).first::<User>(conn){
        Ok(user) => {
            // Verify the hashed password
            let parsed_hash = PasswordHash::new(&user.password_hash).expect("Error parsing password hash");
            let argon2 = Argon2::default();
            Ok(argon2.verify_password(login_password.as_bytes(), &parsed_hash).is_ok())
        }
        // User not found
        Err(diesel::result::Error::NotFound) => Ok(false),
        // Any other errors 
        Err(e) => Err(e),
    }
}