-- clients table
CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    pf_or_pj BOOLEAN NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    cpf_cnpj VARCHAR(255) NOT NULL,
    rg VARCHAR(20),
    cellphone VARCHAR(255) NOT NULL,
    phone VARCHAR(255),
    login VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    mikrotik_id INTEGER,
    plan_id INTEGER,
    address_id INTEGER,
    FOREIGN KEY (address_id) REFERENCES addresses(id),
    FOREIGN KEY (mikrotik_id) REFERENCES mikrotik(id),
    FOREIGN KEY (plan_id) REFERENCES plans(id)
);

-- addresses table
CREATE TABLE addresses (
    id SERIAL PRIMARY KEY,
    cep CHAR(9) NOT NULL,
    street VARCHAR(255) NOT NULL,
    number VARCHAR(255) NOT NULL,
    neighborhood VARCHAR(255) NOT NULL,
    complement VARCHAR(255),
    city VARCHAR(255) NOT NULL,
    state VARCHAR(2) NOT NULL,
    ibge_code VARCHAR(7) NOT NULL
);

-- plans table
CREATE TABLE plans (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    value DECIMAL(10, 2) NOT NULL,
    discount DECIMAL(10, 2),
    technology VARCHAR(50),
--will look into it
--    fiscal_data JSON
);

-- mikrotik table
CREATE TABLE mikrotik (
    id SERIAL PRIMARY KEY,
    ip_address VARCHAR(15) NOT NULL,
    name VARCHAR(255) NOT NULL,
    secret VARCHAR(50) NOT NULL,
    max_clients INTEGER,
    available_bandwidth INTEGER
    --pools JSON
);
