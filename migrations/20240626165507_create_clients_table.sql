CREATE TABLE clientes (
    id SERIAL PRIMARY KEY,
    tipo BOOLEAN NOT NULL,
    nome VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL ,
    cpf_cnpj VARCHAR(255) NOT NULL ,
    formatted_cpf_cnpj VARCHAR(255) NOT NULL,
    telefone VARCHAR(255) NOT NULL,
    login VARCHAR(255) NOT NULL UNIQUE,
    senha VARCHAR(255) NOT NULL,
    mikrotik_id INTEGER,
    plano_id INTEGER,
    cep CHAR(9) NOT NULL,
    rua VARCHAR(255) NOT NULL,
    numero VARCHAR(255) NOT NULL,
    bairro VARCHAR(255) NOT NULL,
    complemento VARCHAR(255),
    cidade VARCHAR(255) NOT NULL,
    estado CHAR(2) NOT NULL,
    ibge_code VARCHAR(7) NOT NULL,
    FOREIGN KEY (plano_id) REFERENCES planos(id),
    FOREIGN KEY (mikrotik_id) REFERENCES mikrotik(id)
);