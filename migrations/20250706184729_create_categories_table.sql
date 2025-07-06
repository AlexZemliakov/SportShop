-- Add migration script here
CREATE TABLE categories (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            name TEXT NOT NULL,
                            description TEXT,
                            image_url TEXT
);