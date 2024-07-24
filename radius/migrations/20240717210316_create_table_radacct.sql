CREATE TABLE radacct (
    radacctid BIGSERIAL PRIMARY KEY,
    acctsessionid VARCHAR(64) NOT NULL DEFAULT '',
    acctuniqueid VARCHAR(32) NOT NULL,
    username VARCHAR(64) NOT NULL DEFAULT '',
    groupname VARCHAR(64) NOT NULL DEFAULT '',
    realm VARCHAR(64) DEFAULT '',
    nasipaddress VARCHAR(15) NOT NULL DEFAULT '',
    nasportid VARCHAR(50) DEFAULT NULL,
    nasporttype VARCHAR(32) DEFAULT NULL,
    acctstarttime TIMESTAMP DEFAULT NULL,
    acctupdatetime TIMESTAMP DEFAULT NULL,
    acctstoptime TIMESTAMP DEFAULT NULL,
    acctinterval INTEGER DEFAULT NULL,
    acctsessiontime INTEGER DEFAULT NULL,
    acctauthentic VARCHAR(32) DEFAULT NULL,
    connectinfo_start VARCHAR(50) DEFAULT NULL,
    connectinfo_stop VARCHAR(50) DEFAULT NULL,
    acctinputoctets BIGINT DEFAULT NULL,
    acctoutputoctets BIGINT DEFAULT NULL,
    calledstationid VARCHAR(50) NOT NULL DEFAULT '',
    callingstationid VARCHAR(50) NOT NULL DEFAULT '',
    acctterminatecause VARCHAR(32) NOT NULL DEFAULT '',
    servicetype VARCHAR(32) DEFAULT NULL,
    framedprotocol VARCHAR(32) DEFAULT NULL,
    framedipaddress VARCHAR(15) NOT NULL DEFAULT '',
    framedipv6address VARCHAR(45) NOT NULL DEFAULT '',
    framedipv6prefix VARCHAR(45) NOT NULL DEFAULT '',
    framedinterfaceid VARCHAR(44) NOT NULL DEFAULT '',
    delegatedipv6prefix VARCHAR(45) NOT NULL DEFAULT '',
    UNIQUE (acctuniqueid)
);

CREATE INDEX idx_username ON radacct(username);
CREATE INDEX idx_framedipaddress ON radacct(framedipaddress);
CREATE INDEX idx_framedipv6address ON radacct(framedipv6address);
CREATE INDEX idx_framedipv6prefix ON radacct(framedipv6prefix);
CREATE INDEX idx_framedinterfaceid ON radacct(framedinterfaceid);
CREATE INDEX idx_delegatedipv6prefix ON radacct(delegatedipv6prefix);
CREATE INDEX idx_acctsessionid ON radacct(acctsessionid);
CREATE INDEX idx_acctsessiontime ON radacct(acctsessiontime);
CREATE INDEX idx_acctstarttime ON radacct(acctstarttime);
CREATE INDEX idx_acctinterval ON radacct(acctinterval);
CREATE INDEX idx_acctstoptime ON radacct(acctstoptime);
CREATE INDEX idx_nasipaddress ON radacct(nasipaddress);
CREATE INDEX idx_bulk_close ON radacct(acctstoptime, nasipaddress, acctstarttime);