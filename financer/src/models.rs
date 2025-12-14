use super::schema::users;
use super::schema::accounts;
use super::schema::contacts;
use super::schema::transactions;
use super::schema::budgets;
use super::schema::recurring_transactions;
use super::schema::recurring_transfers;
use diesel::{Insertable, Queryable};

#[derive(Debug)]
#[allow(dead_code)]
pub enum AccountType {
    Checking,
    Savings,
}

#[derive(Debug, Clone, Queryable)]
#[allow(dead_code)]
pub struct Transaction {
    pub id: i32,
    pub user_account_id: i32,
    pub contact_id: i32,
    pub amount: f32,
    pub category: String,
    pub date: String, 
    pub amount_cents: i32, 
    pub balance_after: f32, 
}

#[derive(Debug, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub user_account_id: i32,
    pub contact_id: i32,
    pub amount: f32,
    pub category: String,
    pub date: String,
    pub amount_cents: i32, 
    pub balance_after: f32, 
}

#[derive(Debug, Queryable)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub account_type: String,
    pub balance: f32,
    pub user_id: i32,
    pub active: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    pub account_type: &'a str,
    pub balance: f32,
    pub user_id: i32,
}
#[derive(Debug, Queryable)]
#[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Period {
    pub fn to_str(&self) -> &'static str {
        match self {
            Period::Daily => "Daily",
            Period::Weekly => "Weekly",
            Period::Monthly => "Monthly",
            Period::Yearly => "Yearly",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Daily" => Period::Daily,
            "Weekly" => Period::Weekly,
            "Monthly" => Period::Monthly,
            "Yearly" => Period::Yearly,
            _ => Period::Monthly,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    Expense, // spending limit
    Income,  // income target
}

impl TargetType {
    pub fn to_str(&self) -> &'static str {
        match self {
            TargetType::Expense => "Expense",
            TargetType::Income => "Income",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Expense" => TargetType::Expense,
            "Income" => TargetType::Income,
            _ => TargetType::Expense, 
        }
    }
}


#[derive(Debug, Queryable, Clone)]
#[allow(dead_code)]
pub struct Budget {
    pub id: Option<i32>, 
    pub user_id: i32,
    pub category: String,
    pub limit_cents: i32, 
    pub period: String, 
    pub target_type: String,
    pub active: bool, 
    pub updated_at: String, 
}

#[derive(Debug, Insertable)]
#[diesel(table_name = budgets)]
pub struct NewBudget {
    pub user_id: i32,
    pub category: String,
    pub limit_cents: i32,
    pub period: String,
    pub target_type: String,
}

#[derive(Debug, Clone, Queryable)]
#[allow(dead_code)]
pub struct RecurringTransaction {
    pub id: i32,
    pub user_id: i32,
    pub account_id: i32,
    pub contact_id: i32,
    pub amount: f32,
    pub category: String,
    pub next_run_at: String,
    pub frequency: String,
    pub active: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = recurring_transactions)]
pub struct NewRecurringTransaction {
    pub user_id: i32,
    pub account_id: i32,
    pub contact_id: i32,
    pub amount: f32,
    pub category: String,
    pub next_run_at: String,
    pub frequency: String,
}

#[derive(Debug, Clone, Queryable)]
#[allow(dead_code)]
pub struct RecurringTransfer {
    pub id: i32,
    pub user_id: i32,
    pub from_account_id: i32,
    pub to_account_id: i32,
    pub amount: f32,
    pub next_run_at: String,
    pub frequency: String,
    pub active: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = recurring_transfers)]
pub struct NewRecurringTransfer {
    pub user_id: i32,
    pub from_account_id: i32,
    pub to_account_id: i32,
    pub amount: f32,
    pub next_run_at: String,
    pub frequency: String,
}
