-- Add migration script here
CREATE TABLE radippool (
    id SERIAL PRIMARY KEY,
    pool_name VARCHAR(64) NOT NULL,
    framedipaddress INET NOT NULL,
    calledstationid VARCHAR(64),
    callingstationid VARCHAR(64),
    username VARCHAR(64),
    pool_key VARCHAR(64) DEFAULT '0'
);
