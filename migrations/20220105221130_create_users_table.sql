-- Add migration script here
CREATE TABLE users(
    user_id UUID PRIMARY KEY UNIQUE DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);