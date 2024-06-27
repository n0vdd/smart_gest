use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct ClientData {
    pub pf_or_pj: bool,
    pub name: String,
    pub email: String,
    pub cpf_cnpj: String,
    #[serde(flatten)]
    pub endereco: Endereco,
    pub cellphone: String,
    pub login: String,
    pub password: String,
    pub mikrotik_id: Option<i32>,
    pub plan_id: i32
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Endereco {
    pub cep: Cep,
    pub endereco: String,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub ibge: String,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Cep {
    pub cep: String,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Plano {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub download_speed: i32,
    pub upload_speed: i32,
    pub description: String,
}


#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Mikrotik {
    pub id: i32,
    pub name: String,
    pub ip: IpAddr,
    pub secret: String,
    pub max_clientes: Option<i32>, 
    pub user: Option<String>,
    //could store this hashed?
    //i think no, its safer
    //it will be used for ssh and doing the fallback logic from radius
    pub password: Option<String>,
}