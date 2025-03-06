#!/bin/bash
set -e

# Ensure data directory exists
mkdir -p data

# Remove existing database
rm -f data/secure_store.db

# Create new database file with permissions
touch data/secure_store.db
chmod 666 data/secure_store.db

# Run SQLite commands to create schema and sample data
sqlite3 data/secure_store.db < setup_db.sql

echo "Database initialized successfully" 