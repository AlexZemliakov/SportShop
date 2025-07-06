-- Up
CREATE TABLE categories (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            name TEXT NOT NULL,
                            description TEXT,
                            image_url TEXT
);

ALTER TABLE products ADD COLUMN category_id INTEGER REFERENCES categories(id);

-- Down
ALTER TABLE products DROP COLUMN category_id;
DROP TABLE categories;