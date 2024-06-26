-- Add migration script here
CREATE TABLE mikrotik (
    id SERIAL PRIMARY KEY,
    ip_address VARCHAR(15) NOT NULL,
    name VARCHAR(255) NOT NULL,
    secret VARCHAR(50) NOT NULL,
    max_clients INTEGER,
    available_bandwidth INTEGER
    --pools JSON
);
