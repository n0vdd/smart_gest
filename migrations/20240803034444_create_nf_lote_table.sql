-- Add migration script here
CREATE TABLE nf_lote (
    id SERIAL PRIMARY KEY,
    month int NOT NULL,
    year int NOT NULL,
    path VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

