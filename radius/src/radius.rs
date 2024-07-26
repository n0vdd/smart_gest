use std::{env, net::Ipv4Addr, process::{Command, ExitStatus}};

use anyhow::anyhow;
use sqlx::{query, types::ipnetwork::IpNetwork,  Connection, PgConnection };
use tracing::error;

pub struct MikrotikNas {
    //This is ip
    pub nasname: String,
    //Name of the mikrotik
    pub shortname: String,
    pub secret: String,
}



pub async fn create_mikrotik_radius(
    mikrotik: MikrotikNas,
) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(std::env::var("DATABASE_URL").unwrap().as_str()).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;

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
        "") 
    .execute(&mut pool)
    .await.map_err(|e| -> _ {
        error!("Failed to insert Mikrotik into nas: {:?}", e);
        anyhow!("Failed to insert Mikrotik into nas radius table".to_string())
    })?;

    //BUG this maybe fucks the clientes who are authed
    let freeradius_restart = Command::new("systemctl restart freeradius").output().map_err(|e| {
        error!("erro ao reiniciar freeradius: {e}");
        anyhow!("erro ao reiniciar freeradius {e}")
    })?;

    //Check is the restart worked
    if freeradius_restart.status.success() {
        Ok(())
    } else {
        error!("Erro ao reiniciar o radius com systemctl {:?} {:?}",freeradius_restart.stdout,freeradius_restart.stderr);
        panic!("Processo de reiniciar radius nao retornou um sucesso")
    }
}

//TODO check if there is duplicate on the cliente login 
//generates a notification 

pub struct ClienteNas {
    pub username: String,
    pub password: String,
    pub plano_nome: String
}

//TODO add users when register cliente is a success(it should call this function after succes,look at how)
pub async fn add_cliente_radius(cliente:ClienteNas) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;

    //Cria o cliente 
    query!("INSERT INTO radcheck(username,attribute,op,value) VALUES ($1,$2,$3,$4)",
    cliente.username,"Cleartext-Password",":=",&cliente.password).execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert user into radcheck: {:?}", e);
        anyhow!("Failed to insert user into radcheck radius table".to_string())
    })?;

    //adiciona o cliente ao plano 
    query!("INSERT into radusergroup(username,groupname) VALUES ($1,$2)",
    cliente.username,cliente.plano_nome).execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert user into radusergroup: {:?}", e);
        anyhow!("Failed to insert user into radusergroup radius table".to_string())
    })?;

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
//Cria uma pool de ips para o cliente usar
//Um loop para gerar 254 ips
pub async fn create_radius_cliente_pool() -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;

    let mut i = 1;
    //Loop para gerar todos os ips validos
    while i < 255 {
        //Instancia um ip para salvar na db
        let ip = Ipv4Addr::new(100, 64, 0, i);
        let ip = IpNetwork::new(std::net::IpAddr::V4(ip), 24).map_err(|e|{
            error!("Failed to create ip network: {:?}", e);
            anyhow!("Failed to create ip network".to_string())
        })?;
        //Insere o ip na db
        query!("INSERT INTO radippool(pool_name,framedipaddress,calledstationid,callingstationid,username,pool_key) VALUES ($1,$2,$3,$4,$5,$6)",
        "pool_valido",ip,"","","","0").execute(&mut pool).await.map_err(|e| {
            error!("Failed to insert ip into radippool: {:?}", e);
            anyhow!("Failed to insert ip into radippool".to_string())
        })?;

        //Incrementa o ip
        i+=1;
    }

    Ok(())
}

pub async fn create_radius_plano_bloqueado() -> Result<(),anyhow::Error> {
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;


    let mut i = 1;
    //Loop para gerar todos os ips bloqueados
    while i < 255 {
        //Instancia um ip para salvar na db
        let ip = Ipv4Addr::new(100, 127, 0, i);
        let ip = IpNetwork::new(std::net::IpAddr::V4(ip), 24).map_err(|e|{
            error!("Failed to create ip network: {:?}", e);
            anyhow!("Failed to create ip network".to_string())
        })?;
        //Insere o ip na db
        query!("INSERT INTO radippool(pool_name,framedipaddress,calledstationid,callingstationid,username,pool_key) VALUES ($1,$2,$3,$4,$5,$6)",
        "pool_bloq",ip,"","","","0").execute(&mut pool).await.map_err(|e| {
            error!("Failed to insert ip into radippool: {:?}", e);
            anyhow!("Failed to insert ip into radippool".to_string())
        })?;

        //Incrementa o ip
        i+=1;
    }

    //Cria o plano bloqueado com a pool de clientes bloqueados
    query!("INSERT INTO radgroupcheck(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    "BLOQUEADO","Pool-Name",":=","pool_bloq").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupcheck: {:?}", e);
        anyhow!("Failed to insert plano into radgroupcheck Pool-Name".to_string())
    })?;

    query!("INSERT INTO radgroupcheck(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    "BLOQUEADO","Service-Type","==","Framed-User").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupcheck: {:?}", e);
        anyhow!("Failed to insert plano into radgroupcheck Service-Type".to_string())
    })?;
    query!("INSERT INTO radgroupcheck(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    "BLOQUEADO","Framed-Protocol",":=","PPP").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupcheck: {:?}", e);
        anyhow!("Failed to insert plano into radgroupcheck Framed-Protocol".to_string())
    })?;
    //Isso pode ser menor,usado para checar a conexao com o cliente
    query!("INSERT INTO radgroupreply(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    "BLOQUEADO","Acct-Interim-Interval",":=","300").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupreply: {:?}", e);
        anyhow!("Failed to insert plano into radgroupreply Acct-Interim-Interval".to_string())
    })?;

    //Seta um limite de internet tao pequeno que se torna impossivel de utilizar o servico
    query!("INSERT INTO radgroupreply(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    "BLOQUEADO","Mikrotik-Rate-Limit",":=","1k/1k").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupreply: {:?}", e);
        anyhow!("Failed to insert plano into radgroupreply Mikrotik-Rate-Limit".to_string())
    })?;

    Ok(())
}

//TODO talvez valha a pena ter uma pool de ips bloqueados
//embora acho que nao seja necessario posso so remover os clientes que nao pagaram,inves de colocar em um plano sem internet
//nao vejo o porque de fazer isso

//TODO disable users if there is no payment confirmed from webhook after 12 days of the month
//UPDATE `radusergroup` SET `groupname` = 'BLOQUEADO' WHERE `username` LIKE 'jose@provedor.com';
//Coloca o cliente como bloqueado(plano sem acesso real a internet)
pub async fn bloqueia_cliente(nome:String) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;

    query!("UPDATE radusergroup SET groupname = 'BLOQUEADO' WHERE username LIKE $1",
    nome).execute(&mut pool).await.map_err(|e| {
        error!("Failed to update user into radusergroup: {:?}", e);
        anyhow!("Failed to update user into radusergroup radius table".to_string())
    })?;

    Ok(())
}

//Recebe o nome do cliente
//Checa se o cliente esta bloqueado
//Retorna true casao ele esteja
pub async fn checa_cliente_bloqueado(nome:&str) -> Result<bool, anyhow::Error> {
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;


    let result = query!("SELECT groupname FROM radusergroup WHERE username LIKE $1",nome).fetch_one(&mut pool).await.map_err(|e| {
        error!("Failed to select user from radusergroup: {:?}", e);
        anyhow!("Failed to select user from radusergroup radius table".to_string())
    })?;

    if result.groupname == "BLOQUEADO" {
        return Ok(true);
    }

    Ok(false)
}

//Recebe o nome do cliente e o plano que ele deve ser colocado
//Atualiza o cliente para o plano correto,liberando o acesso a internet
pub async fn desbloqueia_cliente(nome:&str,plano:String) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await.map_err(|e| {
        error!("Erro ao conectar a db {:?}",e);
        anyhow!("Erro a conectar a radius_db")
    })?;

    //Volta o cliente para o plano quando for confirmado o pagamento
    query!("UPDATE radusergroup SET groupname = $1 WHERE username LIKE $2",
    plano,nome).execute(&mut pool).await.map_err(|e| {
        error!("Failed to update user into radusergroup: {:?}", e);
        anyhow!("Failed to update user into radusergroup radius table".to_string())
    })?;

    Ok(())
}


//TODO enable users, this will be used to enable users that are disabled(when payment is confirmed from webhook(after day 12 of the month)


//TODO cria plano para o usuario

pub struct PlanoRadiusDto {
    pub nome:String,
    pub velocidade_up:i32,
    pub velocidade_down:i32
}

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
//This is called on every added plano on smart_gest
//The plano is used for creating the users with consistent settings
//Just adds the limit and the cliente ip pool
pub async fn create_radius_plano(plano:PlanoRadiusDto) -> Result<(), anyhow::Error> {
    //This solves it
    //BUG this code would be run from another env maybe?
    //Maybe it is not so performatic, needs to keep recreating connections
    //TODO maybe there is a way to use a pool, i dont think there is
    let mut pool = PgConnection::connect(env!("DATABASE_URL")).await
        .map_err(|e| -> _ {
            error!("Failed to create connection: {:?}", e);
            anyhow!("Failed to create connection".to_string())
    })?;

    query!("INSERT INTO radgroupcheck(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    plano.nome,"Service-Type","==","Framed-User").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupcheck: {:?}", e);
        anyhow!("Failed to insert plano into radgroupcheck Service-Type".to_string())
    })?;

    query!("INSERT INTO radgroupcheck(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    plano.nome,"Framed-Protocol",":=","PPP").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupcheck: {:?}", e);
        anyhow!("Failed to insert plano into radgroupcheck Framed-Protocol".to_string())
    })?;

    //Adiciona o pool de ips validos ao plano
    query!("INSERT INTO radgroupcheck(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    plano.nome,"Pool-Name",":=","ips_validos").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupcheck: {:?}", e);
        anyhow!("Failed to insert plano into radgroupcheck Pool-Name".to_string())
    })?;

    //Isso pode ser menor,usado para checar a conexao com o cliente 
    query!("INSERT INTO radgroupreply(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    plano.nome,"Acct-Interim-Interval",":=","300").execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupreply: {:?}", e);
        anyhow!("Failed to insert plano into radgroupreply Acct-Interim-Interval".to_string())
    })?;

    //TODO adicionar um burst com o dobro da velocidade por 1 minuto
    //adiciona o limite de velocidade ao plano
    query!("INSERT INTO radgroupreply(groupname,attribute,op,value) VALUES ($1,$2,$3,$4)",
    plano.nome,"Mikrotik-Rate-Limit",":=",format!("{}/{}",plano.velocidade_up,plano.velocidade_down)).execute(&mut pool).await.map_err(|e| {
        error!("Failed to insert plano into radgroupreply: {:?}", e);
        anyhow!("Failed to insert plano into radgroupreply Mikrotik-Rate-Limit".to_string())
    })?;

    //TODO add the mikrotik ip as the dns server

    Ok(())
}