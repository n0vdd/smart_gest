-- Add migration script here
-- table contratos will have the name, the template name and the path of the saved file
CREATE TABLE contratos_templates (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL UNIQUE,
    path VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE contratos (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL,
    path VARCHAR(255) NOT NULL,
    template_id INTEGER NOT NULL,
    cliente_id INTEGER NOT NULL,
    FOREIGN KEY (template_id) REFERENCES contratos_templates(id),
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE SET NULL
);


--link contrato from planos table to contratos table
-- add foreign key constraint to contrato column on planos table linking it to the id column on contratos table
ALTER TABLE planos ADD FOREIGN KEY (contrato_template_id) REFERENCES contratos_templates(id);


