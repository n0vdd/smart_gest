pub mod radius;

use std::{env, path::Path};
use dotenv::dotenv;

use log::error;
use sqlx::{migrate::{Migrate, Migrator}, MySql, MySqlPool, Pool};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

pub async fn create_mysql_pool() -> Result<Pool<MySql>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").map_err(|e| -> _ {
        error!("DATABASE_URL must be set: {:?}", e);
        DbError::DatabaseError(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "DATABASE_URL must be set"))) 
    }).expect("DATABASE_URL must be set");

    let pool = MySqlPool::connect(&database_url).await
        .map_err(|e| -> _ {
            error!("Failed to create pool: {:?}", e);
            DbError::DatabaseError(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create pool")))
        })?;
    
    //get migrations
    //TODO check if this will overide the db on app restart
    //it should not
    let migrator = Migrator::new(Path::new("./radius/migrations")).await.map_err(|e| {
        error!("Failed to create migrator: {:?}", e);
        DbError::DatabaseError(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create migrator")))
    })?;

    //aply migrations
    migrator.run(&pool).await.map_err(|e| {
        error!("Failed to run migrations: {:?}", e);
        DbError::DatabaseError(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to run migrations")))
    })?;

    Ok(pool)
}

