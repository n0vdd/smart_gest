-- Add migration script here
-- dici will have its id,path,created_at

CREATE TABLE dici (
    id SERIAL PRIMARY KEY,
    path VARCHAR(255) NOT NULL,
    reference_date DATE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);