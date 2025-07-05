-- Таблица категорий
CREATE TABLE categories (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            name TEXT NOT NULL,
                            description TEXT,
                            image_url TEXT
);

-- Таблица товаров
CREATE TABLE products (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          category_id INTEGER REFERENCES categories(id),
                          name TEXT NOT NULL,
                          description TEXT,
                          price REAL NOT NULL,
                          image_url TEXT,
                          stock INTEGER NOT NULL DEFAULT 0,
                          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                          updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Таблица пользователей
CREATE TABLE users (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       telegram_id INTEGER UNIQUE,
                       username TEXT,
                       first_name TEXT,
                       last_name TEXT,
                       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Таблица заказов
CREATE TABLE orders (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER REFERENCES users(id),
                        status TEXT NOT NULL DEFAULT 'new', -- new, processing, completed, cancelled
                        total_amount REAL NOT NULL,
                        ton_address TEXT,
                        payment_status TEXT DEFAULT 'pending', -- pending, paid, failed
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        comments TEXT
);

-- Таблица элементов заказа
CREATE TABLE order_items (
                             id INTEGER PRIMARY KEY AUTOINCREMENT,
                             order_id INTEGER REFERENCES orders(id),
                             product_id INTEGER REFERENCES products(id),
                             quantity INTEGER NOT NULL,
                             price_at_order REAL NOT NULL
);

-- Таблица администраторов
CREATE TABLE admins (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT UNIQUE NOT NULL,
                        password_hash TEXT NOT NULL,
                        role TEXT NOT NULL DEFAULT 'manager' -- manager, admin
);