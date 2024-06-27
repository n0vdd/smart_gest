-- Add migration script here
CREATE TABLE mikrotik (
    id SERIAL PRIMARY KEY,
    ip VARCHAR(15) NOT NULL,
    name VARCHAR(255) NOT NULL,
    secret VARCHAR(50) NOT NULL,
    max_clients INTEGER,
    available_bandwidth INTEGER,
    ssh_login VARCHAR(255),
    ssh_password VARCHAR(255)
    --pools JSON
);
