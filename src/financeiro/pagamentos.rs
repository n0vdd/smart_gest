use sqlx::{query, query_as, PgPool};
use time::PrimitiveDateTime;
use tracing::error;

use super::pagamentos_model::{PaymentConfirmed, PaymentConfirmedDto, PaymentReceivedDto};

pub async fn save_paymente_received_to_db(
    pool: &PgPool,
    payment: &PaymentReceivedDto,
) -> Result<i32, anyhow::Error> {
    //Save the payment_received in the db,linkink it to the payment_confirmed id
    match query!(
            "INSERT INTO payment_received (event_id, payment_confirmed, payment_date) VALUES ($1, $2, $3) RETURNING id",
                payment.event_id,
                payment.payment_confirmed,
                payment.payment_date
        ).fetch_one(&*pool)
        .await {
        Ok(id) => Ok(id.id),

        Err(e) => {
            error!("Failed to save payment: {:?}", e);
            Err(anyhow::anyhow!("Erro ao salvar pagamento"))
        }
    }
}

pub async fn find_pagamento_confirmado_by_cliente_id_and_date(
    pool: &PgPool,
    cliente_id: i32, 
    start: PrimitiveDateTime,
    end: PrimitiveDateTime,
) -> Result<Option<PaymentConfirmed>,anyhow::Error> {
    match query_as!(PaymentConfirmed,"SELECT * FROM payment_confirmed where cliente_id = $1 and
    created_at >= $2 and created_at <= $3",cliente_id,start,end)
        .fetch_optional(pool)
        .await
    {
        Ok(pagamentos) => Ok(pagamentos),
        Err(e) => {
            error!("Failed to fetch pagamentos: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar pagamentos"))
        }
    }
}

pub async fn save_payment_confirmed_to_db(
    pool: &PgPool,
    payment: &PaymentConfirmedDto,
) -> Result<(), anyhow::Error> {
    match query!(
        "INSERT INTO payment_confirmed (cliente_id,payment_date,event_id) VALUES ($1,$2,$3)",
        payment.cliente_id,
        payment.payment_date,
        payment.event_id
    )
    .execute(pool)
    .await {
        Ok(_) => Ok(()),

        Err(e) => {
            error!("Failed to save payment: {:?}", e);
            Err(anyhow::anyhow!("Erro ao salvar pagamento"))
        }
    }
}

pub async fn find_pagamento_confirmado_by_payment_date(
    pool: &PgPool,
    payment_date: time::Date,
) -> Result<Option<PaymentConfirmed>, anyhow::Error> {
    match query_as!(PaymentConfirmed, "SELECT * FROM payment_confirmed where payment_date = $1", payment_date)
        .fetch_optional(pool)
        .await
    {
        Ok(pagamentos) => Ok(pagamentos),

        Err(e) => {
            error!("Failed to fetch pagamentos: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar pagamentos"))
        }
    }
}