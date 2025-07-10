CREATE TABLE products (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          name TEXT NOT NULL,
                          description TEXT NOT NULL,
                          price REAL NOT NULL,
                          stock INTEGER NOT NULL DEFAULT 0,
                          image_url TEXT,
                          category_id INTEGER REFERENCES categories(id),
                          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP

);