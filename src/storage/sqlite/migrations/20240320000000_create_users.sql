-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE,
    phone_number TEXT UNIQUE,
    password_hash TEXT,
    full_name TEXT,
    avatar TEXT,
    created_at INTEGER,
    updated_at INTEGER
);

-- Create verification codes table for phone auth
CREATE TABLE IF NOT EXISTS verification_codes (
    id TEXT PRIMARY KEY,
    phone_number TEXT NOT NULL,
    code TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    attempts INTEGER DEFAULT 0,
    verified BOOLEAN DEFAULT FALSE,
    created_at INTEGER
);
