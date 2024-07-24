CREATE TABLE radpostauth (
    id SERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL DEFAULT '',
    pass VARCHAR(64) NOT NULL DEFAULT '',
    reply VARCHAR(32) NOT NULL DEFAULT '',
    authdate TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
