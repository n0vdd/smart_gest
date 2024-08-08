use sqlx::{query, query_as, PgPool};
use tracing::error;

use super::dici_model::{Dici, DiciDto};

pub async fn find_all_dicis(pool: &PgPool) -> Result<Vec<Dici>, anyhow::Error> {
    match query_as!(Dici,"SELECT * FROM dici ORDER BY reference_date DESC")
        .fetch_all(pool)
        .await
    {
        Ok(dicis) => Ok(dicis),
        Err(e) => {
            error!("Failed to fetch dicis: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar dicis"))
        }
    }
}

pub async fn save_dici(pool: &PgPool,dici: &DiciDto) -> Result<(), anyhow::Error> {
    // Save the DICI to the database
    match query!(
        "INSERT INTO dici (path,reference_date) VALUES ($1,$2)",
        dici.path, dici.reference_date
    )
    .execute(&*pool)
    .await {
        Ok(_) => Ok(()),

        Err(e) => {
            error!("Failed to insert DICI: {:?}", e);
            Err(anyhow::anyhow!("Erro ao inserir DICI"))
        }
    }
}