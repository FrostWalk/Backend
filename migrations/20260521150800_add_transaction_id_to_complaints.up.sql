ALTER TABLE complaints
ADD COLUMN transaction_id INTEGER NOT NULL REFERENCES transactions(transaction_id) ON DELETE CASCADE;
