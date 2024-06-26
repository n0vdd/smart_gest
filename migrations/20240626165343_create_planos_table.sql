-- Add migration script here
CREATE TABLE plans (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    value DECIMAL(10, 2) NOT NULL,
    discount DECIMAL(10, 2),
    technology VARCHAR(50)
    --will look into it
    --fiscal_data JSON
);
