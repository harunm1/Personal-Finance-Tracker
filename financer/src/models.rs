use super::schema::users;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable)]
pub struct Contact {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Queryable)]
pub struct Transaction<'a> {
    pub id: i32,
    pub user_account: &'a Account,
    pub contact: &'a Contact,
    pub amount: f64,
}

#[derive(Debug)]
pub enum AccountType {
    Savings,
    Checking,
}

#[derive(Debug, Queryable)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub account_type: AccountType,
    pub balance: f64,
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub email: Option<&'a str>,
}