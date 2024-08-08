use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::PrimitiveDateTime;

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Mikrotik {
    pub id: i32,
    pub nome: String,
    pub ip: String,
    pub secret: String,
    pub max_clientes: Option<i32>, 
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>,
}


#[derive(Deserialize ,Serialize, Debug, FromRow)]
pub struct MikrotikDto {
    pub nome: String,
    pub ip: Ipv4Addr,
    pub secret: String,
    pub max_clientes: Option<i32>
}
