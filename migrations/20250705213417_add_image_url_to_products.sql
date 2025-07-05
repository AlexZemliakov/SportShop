-- Добавляем колонку image_url
ALTER TABLE products ADD COLUMN image_url TEXT;

-- Обновляем существующие записи (опционально)
UPDATE products SET image_url = NULL;