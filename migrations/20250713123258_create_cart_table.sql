-- В файле миграции для таблицы cart (20250713123258_create_cart_table.sql)
CREATE TABLE cart (
                      id INTEGER PRIMARY KEY AUTOINCREMENT,
                      product_id INTEGER NOT NULL,
                      quantity INTEGER NOT NULL,
                      session_id TEXT NOT NULL,
                      FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);