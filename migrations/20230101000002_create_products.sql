CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    price REAL NOT NULL,
    available BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert some sample products
INSERT INTO products (id, name, description, price, available) VALUES
    ('prod-1', 'Encrypted Password Manager', 'Store and manage your passwords securely', 49.99, TRUE),
    ('prod-2', 'VPN Service (1-year)', 'Secure your internet connection with our VPN', 79.99, TRUE),
    ('prod-3', 'Hardware Authentication Token', 'Two-factor authentication device with biometric verification', 129.99, TRUE); 