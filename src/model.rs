use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientData {
    pub name: String,
    pub email: String,
    pub cpf_cnpj: String,
    pub endereco: Endereco,
    pub cellphone: String,
    pub login: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Endereco {
    pub cep: String,
    pub rua: String,
    pub numero: u32,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub ibge: String
}
