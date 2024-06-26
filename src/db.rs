use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

pub async fn create_pool() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await?;
    Ok(pool)
}
