-- Your SQL goes here
ALTER TABLE accounts ADD COLUMN user_id INTEGER NOT NULL DEFAULT 0 REFERENCES users(id);

