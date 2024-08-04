-- Add migration script here
CREATE TABLE nf_lote (
    id SERIAL PRIMARY KEY,
    month int not null,
    year not null,
    path VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
);

