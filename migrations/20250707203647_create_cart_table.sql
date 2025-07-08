CREATE TABLE cart (
                      id INTEGER PRIMARY KEY,
                      product_id INTEGER NOT NULL,
                      quantity INTEGER NOT NULL DEFAULT 1,
                      user_id INTEGER, -- Для будущей аутентификации
                      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                      FOREIGN KEY (product_id) REFERENCES products(id)
);