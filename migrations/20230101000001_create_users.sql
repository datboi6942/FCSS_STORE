CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add a default admin user (password: admin123)
INSERT INTO users (id, username, password_hash, role, created_at) VALUES
    ('usr-1', 'admin', '$2a$12$5VxU9q4jF9pt2I/q1XOBCu2jC1MlJJeZ7GylP4OyVjaZmztIE5iHa', 'admin', CURRENT_TIMESTAMP); 