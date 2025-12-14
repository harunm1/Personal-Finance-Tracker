-- Add a soft-delete flag for accounts.
ALTER TABLE accounts ADD COLUMN active BOOLEAN NOT NULL DEFAULT 1;
