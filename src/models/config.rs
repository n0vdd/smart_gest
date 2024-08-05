use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Provedor {
    pub id: i32,
    pub nome: String,
    pub cnpj: String,
    pub cep: String,
    pub rua: String,
    pub numero: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub telefone: String,
    pub email: String,
    pub observacao: Option<String>,
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>,
}

//Just set the email that will receive the nf lote
#[derive(Debug, Serialize, Deserialize)]
pub struct NfConfig {
    pub id: i32,
    pub contabilidade_email: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfig {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub host: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvedorDto {
    pub nome: String,
    pub cnpj: String,
    pub cep: String,
    pub rua: String,
    pub numero: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub telefone: String,
    pub email: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfigDto {
    pub email: String,
    pub password: String,
    pub host: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NfConfigDto {
    pub contabilidade_email: String
}