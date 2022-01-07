-- Add migration script here
ALTER TABLE users RENAME password TO hash;