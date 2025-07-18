-- Add delivery_address and dialog_active fields to orders table
ALTER TABLE orders ADD COLUMN delivery_address TEXT;
ALTER TABLE orders ADD COLUMN dialog_active BOOLEAN DEFAULT FALSE;
ALTER TABLE orders ADD COLUMN telegram_message_id INTEGER;
