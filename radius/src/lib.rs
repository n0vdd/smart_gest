use std::env;
use dotenv::dotenv;

use log::error;
use sqlx::{MySql, MySqlPool, Pool};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

pub async fn create_mysql_pool() -> Result<Pool<MySql>> {
    dotenv().ok();
    let database_url = env::var("MYSQL_URL").map_err(|e| -> _ {
        error!("DATABASE_URL must be set: {:?}", e);
        DbError::DatabaseError(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "DATABASE_URL must be set"))) 
    }).expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url).await
        .map_err(|e| -> _ {
            error!("Failed to create pool: {:?}", e);
            DbError::DatabaseError(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create pool")))
        })?;
    Ok(pool)
}

