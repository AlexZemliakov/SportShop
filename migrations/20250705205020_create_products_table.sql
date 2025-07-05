-- migrations/YYYYMMDDHHMMSS_create_products_table.sql
CREATE TABLE products (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          name TEXT NOT NULL,
                          price REAL NOT NULL,
                          stock INTEGER NOT NULL DEFAULT 0,

                          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Дополнительно можно создать другие таблицы
CREATE TABLE categories (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            name TEXT NOT NULL,
                            description TEXT
);