-- Recurring transactions and transfers

CREATE TABLE IF NOT EXISTS recurring_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    contact_id INTEGER NOT NULL DEFAULT 0,
    amount REAL NOT NULL,
    category TEXT NOT NULL,
    next_run_at TEXT NOT NULL,
    frequency TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(account_id) REFERENCES accounts(id)
);

CREATE TABLE IF NOT EXISTS recurring_transfers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    from_account_id INTEGER NOT NULL,
    to_account_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    next_run_at TEXT NOT NULL,
    frequency TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(from_account_id) REFERENCES accounts(id),
    FOREIGN KEY(to_account_id) REFERENCES accounts(id)
);
