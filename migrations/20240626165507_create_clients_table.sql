-- Add migration script here
CREATE TABLE clientes (
    id SERIAL PRIMARY KEY,
    tipo BOOLEAN NOT NULL,
    nome VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    cpf_cnpj VARCHAR(255) NOT NULL,
    formatted_cpf_cnpj VARCHAR(255) NOT NULL,
    telefone VARCHAR(255) NOT NULL,
    login VARCHAR(255) NOT NULL,
    senha VARCHAR(255) NOT NULL,
    mikrotik_id INTEGER,
    plano_id INTEGER,
    endereco_id INTEGER,
    FOREIGN KEY (endereco_id) REFERENCES enderecos(id),
    FOREIGN KEY (plano_id) REFERENCES planos(id),
    FOREIGN KEY (mikrotik_id) REFERENCES mikrotik(id)
);

