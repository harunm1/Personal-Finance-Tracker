-- SQLite may not support DROP COLUMN depending on version.
-- This mirrors the up migration for completeness.
ALTER TABLE accounts DROP COLUMN active;
