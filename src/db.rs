use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod};
use tokio_postgres::NoTls;
use dotenv::dotenv;
use std::env;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error")]
    DatabaseError(#[from] tokio_postgres::Error),
}

pub async fn create_pool() -> Result<deadpool_postgres::Pool> {
    dotenv().ok();
    let mut cfg = Config::new();
    cfg.dbname = Some(env::var("DATABASE_URL").expect("DATABASE_URL must be set").to_string());
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    //I need to pass a runtime and tls
    let pool = cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls).expect("erro ao criar pool de conexões com o banco de dados");
    Ok(pool)
}
