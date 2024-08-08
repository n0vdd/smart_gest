use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

impl fmt::Display for Endereco {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}, {}, {}, {}, {}", self.rua, self.numero.clone().unwrap_or_default(),
        self.bairro, self.cidade, self.estado, self.cep.cep)
    }
}

pub struct Endereco {
    pub cep: Cep,
    pub rua: String,
    pub numero: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub ibge: String,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct EnderecoDto {
    pub cep: String,
    pub rua: String,
    pub numero: Option<String>,
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
