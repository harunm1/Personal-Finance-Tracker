-- Your SQL goes here
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE
);

CREATE TABLE contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    user INTEGER NOT NULL,
    FOREIGN KEY(user) REFERENCES users(id)
);

CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    balance REAL NOT NULL
);

CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_account_id INTEGER NOT NULL,
    contact_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    FOREIGN KEY(user_account_id) REFERENCES accounts(id),
    FOREIGN KEY(contact_id) REFERENCES contacts(id)
);
