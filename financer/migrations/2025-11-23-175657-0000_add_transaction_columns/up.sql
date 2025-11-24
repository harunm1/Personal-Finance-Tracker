-- Your SQL goes here
-- Add category and date columns to transactions
ALTER TABLE transactions ADD COLUMN category TEXT NOT NULL DEFAULT 'Uncategorized';
ALTER TABLE transactions ADD COLUMN date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE transactions ADD COLUMN amount_cents INTEGER NOT NULL DEFAULT 0;

-- Migrate existing float amounts to cents (amount * 100)
UPDATE transactions SET amount_cents = CAST(amount * 100 AS INTEGER);

CREATE INDEX idx_transactions_date ON transactions(date);
CREATE INDEX idx_transactions_account ON transactions(user_account_id);