-- Drop tables if they exist
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS users;

-- Create tables
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    price REAL NOT NULL,
    available BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (product_id) REFERENCES products(id)
);

CREATE TABLE transactions (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL,
    amount REAL NOT NULL,
    status TEXT NOT NULL,
    payment_method TEXT NOT NULL,
    session_id TEXT NOT NULL, 
    currency TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id)
);

-- Add sample data
INSERT INTO users (id, username, password_hash, role, created_at) 
VALUES ('usr-admin1', 'admin', '$2b$10$EgZrLA1Y8AJpR.UnT8a1j.SRRgxzI9iqFadY5lm5eJ87ks5Rm/2Uy', 'admin', CURRENT_TIMESTAMP);

INSERT INTO users (id, username, password_hash, role, created_at) 
VALUES ('usr-user1', 'user', '$2b$10$EgZrLA1Y8AJpR.UnT8a1j.SRRgxzI9iqFadY5lm5eJ87ks5Rm/2Uy', 'user', CURRENT_TIMESTAMP);

INSERT INTO products (id, name, description, price, available, created_at) 
VALUES ('prod-1', 'Premium Widget', 'High-quality widget with advanced features', 99.99, true, CURRENT_TIMESTAMP);

INSERT INTO products (id, name, description, price, available, created_at) 
VALUES ('prod-2', 'Basic Widget', 'Affordable widget with essential features', 49.99, true, CURRENT_TIMESTAMP);

INSERT INTO orders (id, user_id, product_id, status, created_at) 
VALUES ('ord-1', 'usr-user1', 'prod-1', 'completed', CURRENT_TIMESTAMP);

INSERT INTO orders (id, user_id, product_id, status, created_at) 
VALUES ('ord-2', 'usr-user1', 'prod-2', 'pending', CURRENT_TIMESTAMP);

INSERT INTO transactions (id, order_id, amount, status, payment_method, session_id, currency, created_at) 
VALUES ('txn-sample1', 'ord-1', 29.99, 'completed', 'credit_card', 'sess-sample1', 'USD', CURRENT_TIMESTAMP); 