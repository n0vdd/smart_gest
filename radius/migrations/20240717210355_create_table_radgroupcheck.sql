-- Add migration script here

CREATE TABLE radgroupcheck (
    id int(11) unsigned NOT NULL auto_increment,
    groupname varchar(64) NOT NULL default '',
    attribute varchar(64)  NOT NULL default '',
    op char(2) NOT NULL DEFAULT '==',
    value varchar(253)  NOT NULL default '',
    PRIMARY KEY  (id),
    KEY groupname (groupname(32))
);
