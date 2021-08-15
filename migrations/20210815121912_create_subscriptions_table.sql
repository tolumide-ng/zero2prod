-- Add migration script here
-- Create subscriptions table
CREATE TABLE subscriptions(
    id uuid NOT NULL UNIQUE,
    PRIMARY KEY (id),
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
);
