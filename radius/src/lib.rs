use std::{net::Ipv4Addr,  process::Command};

use anyhow::{anyhow, Context};
use sqlx::{query, types::ipnetwork::IpNetwork,  Connection, PgConnection };
use tracing::{error, warn};

#[derive(Debug)]
pub struct MikrotikNas {
    //This is ip
    pub nasname: String,
    //Name of the mikrotik
    pub shortname: String,
    pub secret: String,
}

const DATABASE_URL: &str = "postgres://radius:radpass@localhost:5434/radius";

//Insert mikrotik into nas  table
//with this and the radius server set on the mikrotik, radius should manage its users already
//After adding the mikrotik to the nas table, the freeradius is restarted
///!there is a need to create a entry on sudoers.d/(user running the server) file so that restarting freeradius doesnt needs a password
/// %user ALL= NOPASSWD: /bin/systemctl restart freeradius 
/// TODO radaius should use the table nas to authenticate the mikrotik,not clients.conf
pub async fn create_mikrotik_radius(mikrotik: MikrotikNas) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    let opt_mikrotik = query!("SELECT * FROM nas WHERE nasname LIKE $1", mikrotik.nasname)
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select Mikrotik from nas radius table")?;

    if opt_mikrotik.is_some() {
        warn!("Mikrotik ja existe na base de dados do radius {:?}", mikrotik);
        return Err(anyhow!("Mikrotik ja existe na base de dados do radius"));
    }

    query!(
        "INSERT INTO nas(nasname, shortname, type, ports, secret, server, community, description)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        mikrotik.nasname,
        mikrotik.shortname,
        "other",
        Option::<i32>::None,
        mikrotik.secret,
        Option::<&str>::None,
        Option::<&str>::None,
        ""
    )
    .execute(&mut pool)
    .await
    .context("Failed to insert Mikrotik into nas radius table")?;

    let freeradius_restart = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("freeradius")
        .output()
        .context("erro ao reiniciar freeradius")?;

    if freeradius_restart.status.success() {
        Ok(())
    } else {
        error!(
            "Erro ao reiniciar o radius com systemctl {:?} {:?}",
            freeradius_restart.stdout, freeradius_restart.stderr
        );
        Err(anyhow!("Processo de reiniciar radius nao retornou um sucesso"))
    }
}

pub async fn delete_mikrotik_radius(nasname: String) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    query!("DELETE FROM nas WHERE nasname = $1", nasname)
        .execute(&mut pool)
        .await
        .context("Failed to delete Mikrotik from nas radius table")?;

    let freeradius_restart = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("freeradius")
        .output()
        .context("erro ao reiniciar freeradius")?;

    if freeradius_restart.status.success() {
        Ok(())
    } else {
        error!(
            "Erro ao reiniciar o radius com systemctl {:?} {:?}",
            freeradius_restart.stdout, freeradius_restart.stderr
        );
        Err(anyhow!("Processo de reiniciar radius nao retornou um sucesso"))
    }
}

#[derive(Debug)]
pub struct ClienteNas {
    pub username: String,
    pub password: String,
    pub plano_nome: String
}

//gets the login,password and plano name from the cliente
//add the cliente name and pass as a radius user
//add the cliente to the group of its plano(already with bandiwth limitation and shit)
pub async fn add_cliente_radius(cliente: ClienteNas) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro a conectar a radius_db")?;

    let opt_cliente = query!("SELECT * FROM radcheck WHERE username LIKE $1", cliente.username)
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select user from radcheck radius table")?;

    if opt_cliente.is_some() {
        warn!("Cliente ja existe na base de dados do radius");
        return Err(anyhow!("Cliente ja existe na base de dados do radius"));
    }

    query!(
        "INSERT INTO radcheck(username,attribute,op,value) VALUES ($1,$2,$3,$4)",
        cliente.username,
        "Cleartext-Password",
        ":=",
        &cliente.password
    )
    .execute(&mut pool)
    .await
    .context("Failed to insert user into radcheck radius table")?;

    query!(
        "INSERT into radusergroup(username,groupname) VALUES ($1,$2)",
        cliente.username,
        cliente.plano_nome
    )
    .execute(&mut pool)
    .await
    .context("Failed to insert user into radusergroup radius table")?;

    Ok(())
}

pub async fn delete_cliente_radius(login: String) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro a conectar a radius_db")?;

    let opt_cliente = query!("SELECT * FROM radcheck WHERE username LIKE $1", login)
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select user from radcheck radius table")?;

    if opt_cliente.is_none() {
        warn!("Cliente não existe na base de dados do radius");
        return Err(anyhow!("Cliente não existe na base de dados do radius"));
    }

    query!("DELETE FROM radcheck WHERE username = $1", login)
        .execute(&mut pool)
        .await
        .context("Failed to delete user from radcheck radius table")?;

    query!("DELETE FROM radusergroup WHERE username = $1", login)
        .execute(&mut pool)
        .await
        .context("Failed to delete user from radusergroup radius table")?;

    Ok(())
}

//Cria uma pool de ips para o clientes usarem
//Checa se a pool ja existe antes de adicionar a db(nao existe check na db, so no codigo)
//Um loop para gerar 254 ips para a subnet 100.64.0.0/24
pub async fn create_radius_cliente_pool() -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    let ip_pool = query!("SELECT * FROM radippool WHERE pool_name LIKE 'ips_validos'")
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select ips from radippool")?;

    if ip_pool.is_some() {
        warn!("Pool de ips validos ja foi criada");
        return Err(anyhow!("Pool de ip validos para os clientes ja foi criada"));
    }

    for i in 1..255 {
        let ip = Ipv4Addr::new(100, 64, 0, i);
        let ip = IpNetwork::new(std::net::IpAddr::V4(ip), 24)
            .context("Failed to create ip network")?;

        query!("INSERT INTO radippool(pool_name, framedipaddress, calledstationid, callingstationid, username, pool_key) VALUES ($1, $2, $3, $4, $5, $6)",
               "pool_valido", ip, "", "", "", "0")
            .execute(&mut pool)
            .await
            .context("Failed to insert ip into radippool")?;
    }

    Ok(())
}


//Cria a pool utilizada pelo plano bloqueado
//Cria o plano BLOQUEADO(plano com 1k de velocidade), tornando impossivel a utilizacao do servico
//Esse plano permite bloquear o cliente simplesmente trocando o cliente do seu plano padrao para o plano bloqueado
pub async fn create_radius_plano_bloqueado() -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    let plano_bloqueado = query!("SELECT * FROM radgroupcheck WHERE groupname LIKE 'BLOQUEADO'")
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select plano from radgroupcheck")?;

    if plano_bloqueado.is_some() {
        warn!("Plano bloqueado ja foi criado");
        return Ok(());
    }

    for i in 1..255 {
        let ip = Ipv4Addr::new(100, 65, 0, i);
        let ip = IpNetwork::new(std::net::IpAddr::V4(ip), 24)
            .context("Failed to create ip network")?;

        query!("INSERT INTO radippool(pool_name, framedipaddress, calledstationid, callingstationid, username, pool_key) VALUES ($1, $2, $3, $4, $5, $6)",
               "pool_bloq", ip, "", "", "", "0")
            .execute(&mut pool)
            .await
            .context("Failed to insert ip into radippool")?;
    }

    query!("INSERT INTO radgroupcheck(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           "BLOQUEADO", "Pool-Name", ":=", "pool_bloq")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupcheck Pool-Name")?;

    query!("INSERT INTO radgroupcheck(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           "BLOQUEADO", "Service-Type", "==", "Framed-User")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupcheck Service-Type")?;

    query!("INSERT INTO radgroupcheck(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           "BLOQUEADO", "Framed-Protocol", ":=", "PPP")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupcheck Framed-Protocol")?;

    query!("INSERT INTO radgroupreply(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           "BLOQUEADO", "Acct-Interim-Interval", ":=", "300")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupreply Acct-Interim-Interval")?;

    query!("INSERT INTO radgroupreply(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           "BLOQUEADO", "Mikrotik-Rate-Limit", ":=", "1k/1k")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupreply Mikrotik-Rate-Limit")?;

    Ok(())
}

//Recebe o login do cliente para bloquear
//Coloca o cliente como bloqueado(plano sem acesso real a internet),ao trocalo de seu grupo padrao para o grupo bloqueado
pub async fn bloqueia_cliente_radius(login: &str) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    query!("UPDATE radusergroup SET groupname = 'BLOQUEADO' WHERE username LIKE $1", login)
        .execute(&mut pool)
        .await
        .context("Failed to update user into radusergroup radius table")?;

    Ok(())
}

//Recebe o login do cliente
//Checa se o cliente faz parte do grupo bloqueado
//Retorna true casao ele faca 
pub async fn checa_cliente_bloqueado_radius(login: &str) -> Result<bool, anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    let result = query!("SELECT groupname FROM radusergroup WHERE username LIKE $1", login)
        .fetch_one(&mut pool)
        .await
        .context("Failed to select user from radusergroup radius table")?;

    Ok(result.groupname == "BLOQUEADO")
}

//Recebe o login do cliente e o plano que ele deve ser colocado
//Atualiza o cliente para o plano correto,liberando o acesso a internet
pub async fn desbloqueia_cliente(login: &str, plano: String) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    query!("UPDATE radusergroup SET groupname = $1 WHERE username LIKE $2", plano, login)
        .execute(&mut pool)
        .await
        .context("Failed to update user into radusergroup radius table")?;

    Ok(())
}


#[derive(Debug)]
pub struct PlanoRadiusDto {
    pub nome:String,
    pub velocidade_up:i32,
    pub velocidade_down:i32
}

//This is called on every added plano on smart_gest
//The plano is used for creating the users with consistent settings
//Just adds the limit and the cliente ip pool
//Receives the name of the plano and the connection speed
//Creates the plano on the radius server, its all linked by its name, uses the valid ip pool
//the one created by: fn create_radius_cliente_pool()
pub async fn create_radius_plano(plano: PlanoRadiusDto) -> Result<(), anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    let plano_criado = query!("SELECT * FROM radgroupcheck WHERE groupname LIKE $1", plano.nome)
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select plano from radgroupcheck")?;

    if plano_criado.is_some() {
        warn!("Plano ja foi criado {:?}", plano);
        return Err(anyhow!("Plano ja foi criado {:?}", plano));
    }

    query!("INSERT INTO radgroupcheck(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           plano.nome, "Service-Type", "==", "Framed-User")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupcheck Service-Type")?;

    query!("INSERT INTO radgroupcheck(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           plano.nome, "Framed-Protocol", ":=", "PPP")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupcheck Framed-Protocol")?;

    query!("INSERT INTO radgroupcheck(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           plano.nome, "Pool-Name", ":=", "ips_validos")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupcheck Pool-Name")?;

    query!("INSERT INTO radgroupreply(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           plano.nome, "Acct-Interim-Interval", ":=", "300")
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupreply Acct-Interim-Interval")?;

    query!("INSERT INTO radgroupreply(groupname, attribute, op, value) VALUES ($1, $2, $3, $4)",
           plano.nome, "Mikrotik-Rate-Limit", ":=", format!("{}m/{}m", plano.velocidade_up, plano.velocidade_down))
        .execute(&mut pool)
        .await
        .context("Failed to insert plano into radgroupreply Mikrotik-Rate-Limit")?;

    Ok(())
}

pub async fn delete_radius_plano(plano_nome: &str) -> Result<(),anyhow::Error> {
    let mut pool = PgConnection::connect(DATABASE_URL)
        .await
        .context("Erro ao conectar a radius_db")?;

    let plano_existe = query!("SELECT * FROM radgroupcheck WHERE groupname LIKE $1", plano_nome)
        .fetch_optional(&mut pool)
        .await
        .context("Failed to select plano from radgroupcheck")?;

    if plano_existe.is_none() {
        warn!("Plano não encontrado {:?}", plano_nome);
        return Err(anyhow!("Plano não encontrado {:?}", plano_nome));
    }

    query!("DELETE FROM radgroupcheck WHERE groupname = $1", plano_nome)
        .execute(&mut pool)
        .await
        .context("Failed to delete plano from radgroupcheck")?;

    query!("DELETE FROM radgroupreply WHERE groupname = $1", plano_nome)
        .execute(&mut pool)
        .await
        .context("Failed to delete plano from radgroupreply")?;

    Ok(())
}