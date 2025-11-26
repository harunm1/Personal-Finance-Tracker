-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_transactions_date;
DROP INDEX IF EXISTS idx_transactions_account;
-- Note: SQLite doesn't support DROP COLUMN easily, so we leave columns