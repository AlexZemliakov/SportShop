-- migrations/20250707120000_create_cart_table.sql
CREATE TABLE IF NOT EXISTS cart (
                                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                                    product_id INTEGER NOT NULL,
                                    quantity INTEGER NOT NULL,
                                    user_id INTEGER,
                                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                                    FOREIGN KEY (product_id) REFERENCES products(id)
    );