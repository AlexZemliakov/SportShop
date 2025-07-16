-- В файле migrations/20250706184900_create_orders_table.sql
CREATE TABLE orders (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id BIGINT NOT NULL,
                        total_amount REAL NOT NULL,
                        status TEXT NOT NULL,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);