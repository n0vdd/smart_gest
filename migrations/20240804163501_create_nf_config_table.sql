-- Add migration script here
create TABLE nf_config (
    id SERIAL PRIMARY KEY,
    contabilidade_email TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER nf_config_update_updated_at
BEFORE UPDATE ON nf_config
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();