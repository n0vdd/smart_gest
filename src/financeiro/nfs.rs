use sqlx::{query, query_as, PgPool};
use tracing::error;

use super::nfs_model::{NfDto, NfLote, NfLoteDto};

pub async fn find_nf_lote_by_id(pool: &PgPool, id: i32) -> Result<NfLote, anyhow::Error> {
    match query_as!(NfLote,"SELECT * FROM nf_lote WHERE id = $1",id)
        .fetch_one(pool)
        .await
    {
        Ok(nf) => Ok(nf),
        Err(e) => {
            error!("Failed to fetch nf lote: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar nf lote"))
        }
    }
}

pub async fn find_all_nfs_lotes(pool: &PgPool) -> Result<Vec<NfLote>, anyhow::Error> {
    match query_as!(NfLote,"SELECT * FROM nf_lote")
        .fetch_all(pool)
        .await
    {
        Ok(nfs) => Ok(nfs),
        Err(e) => {
            error!("Failed to fetch nfs: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar nfs"))
        }
    }
}

//TODO download nota fiscal

pub async fn save_nf_lote(pool: &PgPool,nf_lote: &NfLoteDto) -> Result<(), anyhow::Error> {
    // Save the NF Lote to the database
    match query!(
        "INSERT INTO nf_lote (path,month,year) VALUES ($1,$2,$3)",
        nf_lote.path, nf_lote.month, nf_lote.year
    )
    .execute(&*pool)
    .await {
        Ok(_) => Ok(()),

        Err(e) => {
            error!("Failed to insert NF Lote: {:?}", e);
            Err(anyhow::anyhow!("Erro ao inserir NF Lote"))
        }
    }
}

pub async fn save_nf(pool:&PgPool,nf:&NfDto) -> Result<(), anyhow::Error> {
    match query!("INSERT INTO nfs (path, payment_received_id, sent) VALUES ($1, $2, $3)",
        nf.path,nf.payment_received_id, nf.sent)
            .execute(pool).await {
        Ok(_) => Ok(()),

        Err(e) => {
            error!("Failed to insert NF: {:?}", e);
            Err(anyhow::anyhow!("Erro ao salvar NF na db"))
        }
    }
}