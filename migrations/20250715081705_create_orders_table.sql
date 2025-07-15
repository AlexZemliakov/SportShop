CREATE TABLE orders (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        session_id TEXT NOT NULL,
                        total REAL NOT NULL,
                        address TEXT NOT NULL,
                        created_at TEXT NOT NULL,  -- или DATETIME
                        FOREIGN KEY (session_id) REFERENCES cart(session_id)
);

-- Создание таблицы элементов заказа (как у вас)
CREATE TABLE order_items (
                             id INTEGER PRIMARY KEY AUTOINCREMENT,
                             order_id INTEGER NOT NULL,
                             product_id INTEGER NOT NULL,
                             quantity INTEGER NOT NULL,
                             FOREIGN KEY (order_id) REFERENCES orders(id),
                             FOREIGN KEY (product_id) REFERENCES products(id)
);