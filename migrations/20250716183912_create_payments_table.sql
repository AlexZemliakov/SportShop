-- Up
CREATE TABLE payments (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          order_id TEXT NOT NULL UNIQUE,
                          user_id BIGINT NOT NULL,
                          amount REAL NOT NULL,
                          wallet_address TEXT NOT NULL,
                          status TEXT NOT NULL DEFAULT 'pending',
                          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                          ton_payment_id TEXT
);

-- Down
DROP TABLE payments;