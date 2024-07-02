-- Add migration script here
CREATE TABLE planos (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL UNIQUE,
    descricao TEXT,
    valor REAL NOT NULL,
    velocidade_up INTEGER NOT NULL,
    velocidade_down INTEGER NOT NULL,
    -- referenciar a tabela de contratos
    -- opcoes(fibra,fibra+voip(ilimitado e limitado nao precisa diferenciar))
    -- TODO quem pega fibra+voip tem gerado o contrato de fibra e de voip
    contrato_template_id INTEGER NOT NULL,
    -- possot ter um plano para boleto e outro cartao
    -- caso cartao de credito -10,00
    -- desconto DECIMAL(10, 2),
    --fibra,telefone,email...
    tecnologia VARCHAR(50)
    --will look into it
    --fiscal_data JSON
);
