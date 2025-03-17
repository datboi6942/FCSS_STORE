-- Update order_items table schema to match what the code expects
ALTER TABLE order_items ADD COLUMN product_name TEXT;
ALTER TABLE order_items RENAME COLUMN order_number TO order_id; 