-- Your SQL goes here
CREATE TABLE budgets (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  category TEXT NOT NULL,
  limit_cents INTEGER NOT NULL,
  period TEXT NOT NULL,
  target_type TEXT NOT NULL,
  active BOOLEAN NOT NULL DEFAULT 1,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_budgets_user_id ON budgets(user_id);