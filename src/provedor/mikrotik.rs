use anyhow::Context;
use radius::{create_mikrotik_radius, delete_mikrotik_radius, MikrotikNas};
use sqlx::{query, query_as, PgPool};
use tracing::error;

use super::mikrotik_model::{Mikrotik, MikrotikDto};

//TODO pagination and shit?
pub async fn find_all_mikrotiks(pool:&PgPool) -> Result<Vec<Mikrotik>,anyhow::Error> {
    match query_as!(
        Mikrotik,
        "SELECT * FROM mikrotik"
    )
    .fetch_all(pool)
    .await {
        Ok(mikrotiks) => Ok(mikrotiks),
        Err(e) => {
            error!("Failed to fetch mikrotiks: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar mikrotiks"))
        }
    }
}

pub async fn update_mikrotik_db(mikrotik:&Mikrotik,pool:&PgPool) -> Result<Mikrotik,anyhow::Error> {
    let mikrotik = query_as!(Mikrotik,
        "UPDATE mikrotik SET nome = $1, ip = $2, secret = $3, max_clientes = $4 WHERE id = $5 RETURNING *",
        mikrotik.nome,
        mikrotik.ip,
        mikrotik.secret,
        mikrotik.max_clientes,
        mikrotik.id
    ).fetch_one(&*pool).await.context("Erro ao atualizar mikrotik")?;

    Ok(mikrotik)
}

pub async fn delete_mikrotik_db(id:i32,pool:&PgPool) -> Result<(),anyhow::Error> {
    let nome = query!("SELECT ip from mikrotik where id = $1",id)
        .fetch_one(&*pool)
        .await
        .context("Erro ao buscar nome do mikrotik")?;

    delete_mikrotik_radius(nome.ip).await.context("Erro ao deletar mikrotik no radius")?;

    query!(
        "DELETE FROM mikrotik WHERE id = $1",
        id
    ).execute(&*pool).await.context("Erro ao deletar mikrotik")?;

    Ok(())
}

pub async fn save_mikrotik(mikrotik:&MikrotikDto,pool:&PgPool) -> Result<(),anyhow::Error> {
    let mikrotik_nas = MikrotikNas {
        nasname: mikrotik.ip.to_string(),
        shortname: mikrotik.nome.clone(),
        secret: mikrotik.secret.clone(),
    };

    create_mikrotik_radius(mikrotik_nas).await?;

    query!(
        "INSERT INTO mikrotik (nome,ip,secret,max_clientes) VALUES ($1,$2,$3,$4)",
        mikrotik.nome,
        mikrotik.ip.to_string(),
        mikrotik.secret,
        mikrotik.max_clientes
    ).execute(&*pool).await.context("Erro ao salvar mikrotik")?;

    Ok(())
}

pub async fn find_mikrotik_by_id(id:i32,pool:&PgPool) -> Result<Mikrotik,anyhow::Error> {
    let mikrotik = query_as!(Mikrotik,
        "SELECT * FROM mikrotik WHERE id = $1",
        id
    ).fetch_one(&*pool).await.context("Erro ao buscar mikrotik")?;

    Ok(mikrotik)
}