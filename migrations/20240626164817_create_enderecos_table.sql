-- Add migration script here
-- Create enderecos table
CREATE TABLE enderecos (
    id SERIAL PRIMARY KEY,
    cep CHAR(9) NOT NULL,
    rua VARCHAR(255) NOT NULL,
    numero VARCHAR(255) NOT NULL,
    bairro VARCHAR(255) NOT NULL,
    complemento VARCHAR(255),
    cidade VARCHAR(255) NOT NULL,
    estado CHAR(2) NOT NULL,
    ibge_code VARCHAR(7) NOT NULL
);
