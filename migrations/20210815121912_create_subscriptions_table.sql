-- Add migration script here
-- Create subscriptions table
CREATE TABLE subscriptions(
    id UUID PRIMARY KEY UNIQUE DEFAULT uuid_generate_v4(),
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    subscribed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
