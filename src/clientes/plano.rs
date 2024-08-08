use anyhow::Context;
use radius::delete_radius_plano;
use sqlx::{query, query_as, PgPool};
use tracing::error;

use crate::clientes::plano_model::Plano;

use super::plano_model::PlanoDto;

pub async fn delete_plano_by_id(pool:&PgPool,id: i32) -> Result<(),anyhow::Error> {
    let nome = query!(
        "SELECT nome from planos where id = $1",
        id
    ).fetch_one(pool).await.context("Erro ao buscar nome do plano")?.nome;

    delete_radius_plano(&nome).await?;

    query!(
        "DELETE FROM planos WHERE id = $1",
        id 
    ).execute(pool).await.context("Erro ao deletar plano")?; 

    Ok(())
}

//Recebe a id de um cliente
//Utiliza a id do cliente para achar qual o plano associado aquele cliente
//retorna o plano
pub async fn find_plano_by_cliente(pool:&PgPool,cliente_id: i32) -> Result<Plano,anyhow::Error> {
    match query_as!(
        Plano,
        "SELECT planos.* FROM planos INNER JOIN clientes ON planos.id = clientes.plano_id WHERE clientes.id = $1",
        cliente_id
    )
    .fetch_one(pool)
    .await {
        Ok(plano) => Ok(plano),

        Err(e) => {
            error!("Failed to fetch plano: {:?}", e);
            Err(anyhow::anyhow!("Failed to fetch plano with id: {cliente_id} from db"))
        }
    }
}

pub async fn find_all_planos(pool:&PgPool) -> Result<Vec<Plano>,anyhow::Error> {
    match query_as!(
        Plano,
        "SELECT * FROM planos"
    )
    .fetch_all(pool)
    .await {
        Ok(planos) => Ok(planos),
        Err(e) => {
            error!("Failed to fetch planos: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar planos"))
        }
    }
}

pub async fn find_plano_by_id(pool:&PgPool,id: i32) -> Result<Plano,anyhow::Error> {
    match query_as!(
        Plano,
        "SELECT planos.* FROM planos INNER JOIN clientes ON planos.id = clientes.plano_id WHERE clientes.id = $1",
        id 
    )
    .fetch_one(pool)
    .await {
        Ok(plano) => Ok(plano),
        Err(e) => {
            error!("Failed to fetch plano: {:?}", e);
            Err(anyhow::anyhow!("Failed to fetch plano with id: {id} from db"))
        }
    }
}

pub async fn save_plano(pool:&PgPool,plano:&PlanoDto) -> Result<(),anyhow::Error> {
    match query!(
        "INSERT INTO planos (nome, valor, velocidade_up,velocidade_down, descricao,tipo_pagamento, contrato_template_id)
        VALUES ($1, $2, $3, $4, $5, $6,$7)",
        plano.nome,
        plano.valor,
        plano.velocidade_up,
        plano.velocidade_down,
        plano.descricao,
        //easier to save as string than to implemente decode/encode
        plano.tipo_pagamento.to_string(),
        //plano.tecnologia,
        plano.contrato_template_id)
        .execute(pool)
        .await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to save plano: {:?}", e);
                Err(anyhow::anyhow!("Erro ao salvar plano"))
            }
        }
}

pub async fn update_plano_db(pool:&PgPool,plano:&Plano) -> Result<(),anyhow::Error> {
    match query!(
        "UPDATE planos SET nome = $1, valor = $2, velocidade_up = $3, velocidade_down = $4, descricao = $5, tipo_pagamento = $6, contrato_template_id = $7 WHERE id = $8",
        plano.nome,
        plano.valor,
        plano.velocidade_up,
        plano.velocidade_down,
        plano.descricao,
        plano.tipo_pagamento.to_string(),
        plano.contrato_template_id,
        plano.id
    )
    .execute(pool)
    .await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to update plano: {:?}", e);
            Err(anyhow::anyhow!("Erro ao atualizar plano"))
        }
    }
}