-- Add migration script here
--will add audit data for record creation(user_id(created_by),and date of creation(created_by)
--record changes date of change(updated_at) and user_id(update_by)) to the table
--and record deletion date of deletion(deleted_at) and user_id(deleted_by) to the table
--for having deletion i should use soft deletes
-- TODO add created_at and update_at for clientes,mikrotik,planos,contratos
ALTER TABLE clientes
ADD COLUMN created_at TIMESTAMP DEFAULT NOW(),
ADD COLUMN updated_at TIMESTAMP DEFAULT NOW();

ALTER TABLE mikrotik
ADD COLUMN created_at TIMESTAMP DEFAULT NOW(),
ADD COLUMN updated_at TIMESTAMP DEFAULT NOW();

ALTER TABLE planos
ADD COLUMN created_at TIMESTAMP DEFAULT NOW(),
ADD COLUMN updated_at TIMESTAMP DEFAULT NOW();

ALTER TABLE contratos
ADD COLUMN created_at TIMESTAMP DEFAULT NOW(),
ADD COLUMN updated_at TIMESTAMP DEFAULT NOW();

-- Trigger to update updated_at column on table update
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for each table to update updated_at column
CREATE TRIGGER clientes_update_updated_at
BEFORE UPDATE ON clientes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER mikrotik_update_updated_at
BEFORE UPDATE ON mikrotik
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER planos_update_updated_at
BEFORE UPDATE ON planos
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER contratos_update_updated_at
BEFORE UPDATE ON contratos
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
