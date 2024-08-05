-- Add migration script here
create TABLE email_config (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    --this should be text to deal with dns and ipaddr
    host TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER email_config_update_updated_at
BEFORE UPDATE ON email_config
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
