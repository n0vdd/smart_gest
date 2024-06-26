-- Add migration script here
-- Create enderecos table
CREATE TABLE enderecos (
    id SERIAL PRIMARY KEY,
    cep CHAR(9) NOT NULL,
    street VARCHAR(255) NOT NULL,
    number VARCHAR(255) NOT NULL,
    neighborhood VARCHAR(255) NOT NULL,
    complement VARCHAR(255),
    city VARCHAR(255) NOT NULL,
    state CHAR(2) NOT NULL,
    ibge_code VARCHAR(7) NOT NULL
);
