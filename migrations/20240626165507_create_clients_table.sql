-- Add migration script here
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
    FOREIGN KEY (address_id) REFERENCES enderecos(id),
    FOREIGN KEY (plan_id) REFERENCES plans(id),
    FOREIGN KEY (mikrotik_id) REFERENCES mikrotik(id)
);

