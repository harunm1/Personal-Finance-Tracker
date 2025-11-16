use super::schema::users;
use super::schema::accounts;
use super::schema::contacts;
use super::schema::transactions;
use diesel::{Insertable, Queryable};

#[derive(Debug)]
pub enum AccountType {
    Checking,
    Savings,
}

#[derive(Debug, Queryable)]
pub struct Transaction {
    pub id: i32,
    pub user_account_id: i32,
    pub contact_id: i32,
    pub amount: f32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub user_account_id: i32,
    pub contact_id: i32,
    pub amount: f32,
}

#[derive(Debug, Queryable)]
pub struct Contact<'a> {
    pub id: i32,
    pub name: &'a str,
    pub user: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = contacts)]
pub struct NewContact<'a> {
    pub name: &'a str,
    pub user: i32,
}
 
#[derive(Debug, Queryable)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub account_type: String,
    pub balance: f32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    pub account_type: &'a str,
    pub balance: f32,
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
