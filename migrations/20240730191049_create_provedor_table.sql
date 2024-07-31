-- Add migration script here
-- um provedor sera seu nome,endereco,cpf/cnpj,telefone, email, e um campo de observacao
-- o sistema tera so um, acessado pelo campo de configuracao
-- TODO a questao da nota fiscal sera um problema para depois
-- como tornar ela mais generica e afins, tenho que olhar sobre traits e coisas do tipo
CREATE TABLE provedor (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL,
    cpf_cnpj VARCHAR(14) NOT NULL,
    telefone VARCHAR(15) NOT NULL,
    email VARCHAR(255) NOT NULL,
    rua VARCHAR(255) NOT NULL,
    numero VARCHAR(10) NOT NULL,
    complemento VARCHAR(255),
    bairro VARCHAR(255) NOT NULL,
    cidade VARCHAR(255) NOT NULL,
    estado CHAR(2) NOT NULL,
    observacao TEXT
);

