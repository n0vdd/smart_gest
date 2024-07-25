use anyhow::anyhow;
use sqlx::{query, PgPool};
use tracing::error;

pub struct MikrotikNas {
    //This is ip
    pub nasname: String,
    //Name of the mikrotik
    pub shortname: String,
    pub secret: String,
    //I think mikrotik does not have this data
    pub description: String,
}


//TODO will be used for controlling the freeradius db(mysql)
pub async fn insert_mikrotik_to_radius_db(
    pool: &PgPool,
    mikrotik: MikrotikNas,
) -> Result<(), anyhow::Error> {
    query!(
        "INSERT INTO nas(nasname, shortname, type, ports, secret, server, community, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7,$8)",
        mikrotik.nasname,
        mikrotik.shortname,
        "other",
        Option::<i32>::None,
        mikrotik.secret,
        Option::<&str>::None,
        Option::<&str>::None,
        mikrotik.description) 
    .execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to insert Mikrotik into nas: {:?}", e);
        anyhow!("Failed to insert Mikrotik into nas radius table".to_string())
    })?;

    Ok(())
}


pub struct ClienteNas {
    pub username: String,
    pub password: String,
    pub velocidade_up: i32,
    pub velocidade_down: i32
}

//TODO add users when register cliente is a success(it should call this function after succes,look at how)
pub async fn insert_user_db(pool:&PgPool,cliente:ClienteNas) -> Result<(), anyhow::Error> {

    query!("INSERT INTO radcheck(username,attribute,op,value) VALUES ($1,$2,$3,$4)",
    cliente.username,"Cleartext-Password",":=",&cliente.password).execute(pool).await.map_err(|e| {
        error!("Failed to insert user into radcheck: {:?}", e);
        anyhow!("Failed to insert user into radcheck radius table".to_string())
    })?;
    //TODO adiciona o cliente ao plano 
    /*
INSERT INTO `radusergroup` (`username`, `groupname`) VALUES 
('jose@provedor.com', 'PLANO_10MB'); 
     */

    //Dont need this
    query!("INSERT INTO radcheck(username,attribute,op,value) VALUES ($1,$2,$3,$4)",
    cliente.username,"Service-Type",":=","Framed-User").execute(pool).await.map_err(|e| {
        error!("Failed to insert user into radcheck: {:?}", e);
        anyhow!("Failed to insert user into radcheck service-type".to_string())
    })?;

    //Dont need this
    query!("INSERT INTO radcheck(username,attribute,op,value) VALUES ($1,$2,$3,$4)",
    cliente.username,"Framed-Protocol",":=","PPP").execute(pool).await.map_err(|e| {
        error!("Failed to insert user into radcheck: {:?}", e);
        anyhow!("Failed to insert user into radcheck framed-protocol".to_string())
    })?;

    //Dont need this
    //TODO criar um burst de 200mb? necessario alterar no plano/cadastro do plano tambem
    //INSERT INTO `radreply` (`id`, `username`, `attribute`, `op`, `value`) VALUES
//(NULL, 'jose@provedor.com', 'Mikrotik-Rate-Limit', ':=', '10M/50M 20M/100M 10M/50M 120/120 0');
    query!("INSERT INTO radreply(username,attribute,op,value) VALUES ($1,$2,$3,$4)",
    cliente.username,"Mikrotik-Rate-Limit",":=",format!("{}/{}",cliente.velocidade_up,cliente.velocidade_down)).execute(pool).await.map_err(|e| {
        error!("Failed to insert user into radreply: {:?}", e);
        anyhow!("Failed to insert user into radreply with connection limit".to_string())
    })?;

    //TODO adicionar o dns do mikrotik ao cliente
    /*
        INSERT INTO `radreply` (`id`, `username`, `attribute`, `op`, `value`) VALUES
        (NULL, 'jose@provedor.com', 'MS-Primary-DNS-Server', ':=', '1.1.1.1'),
     */

    //TODO adiciona o cliente a pool de ip valido
    /*
INSERT INTO `radcheck` (`id`, `username`, `attribute`, `op`, `value`) VALUES 
(NULL, 'jose@provedor.com', 'Pool-Name', ':=', 'pool_valido');
     */


    Ok(())
}

//TODO criar uma pool com os ips validos
/*
INSERT INTO `radippool` 
(`pool_name`, `framedipaddress`, `calledstationid`, `callingstationid`, `username`, `pool_key`) VALUES
('pool_valido', '180.255.3.128','','','','0'),
('pool_valido', '180.255.3.129','','','','0'),
('pool_valido', '180.255.3.130','','','','0'),
('pool_bloq','100.127.0.0','','','','0'),
('pool_bloq','100.127.0.1','','','','0'),
('pool_bloq','100.127.0.2','','','','0');
*/

//TODO disable users if there is no payment confirmed from webhook after 12 days of the month

//TODO enable users, this will be used to enable users that are disabled(when payment is confirmed from webhook(after day 12 of the month)


//TODO cria plano para o usuario
/*

INSERT INTO `radgroupcheck` (`groupname`, `attribute`, `op`, `value`) VALUES
('PLANO_10MB', 'Service-Type', '==', 'Framed-User'),
('PLANO_10MB', 'Framed-Protocol', ':=', 'PPP'),
('PLANO_10MB', 'Pool-Name', ':=', 'pool_valido');
INSERT INTO `radgroupreply` (`groupname`, `attribute`, `op`, `value`) VALUES
('PLANO_10MB', 'Acct-Interim-Interval', ':=', '300'),
('PLANO_10MB', 'Mikrotik-Rate-Limit', ':=', '5M/10M'),
('PLANO_10MB', 'MS-Primary-DNS-Server', ':=', '9.9.9.9'),
('PLANO_10MB', 'MS-Secondary-DNS-Server', ':=', '149.112.112.112');
*/