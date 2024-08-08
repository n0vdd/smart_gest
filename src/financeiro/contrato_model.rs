use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Struct for storing formatted client data.
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ClienteContractData {
    pub id: i32,
    pub nome: String,
    pub login: String,
    pub rua: String,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub cep: String,
    pub formatted_cpf_cnpj: String,
    pub contrato_template_path: String,
    pub contrato_template_nome: String,
    pub contrato_template_id: i32,
    pub plano_id: Option<i32>
}

//Contratos sao exibidos com o nome
//gerados com o template
//e salvam o caminho do arquivo gerado
#[derive(Serialize, Deserialize, Debug,FromRow)]
struct Contrato {
    id: i32,
    nome: String,
    path: String,
    template_id: i32,
    cliente_id: i32,
}

pub struct ContratoDto {
    pub nome: String,
    pub path: String,
    pub template_id: i32,
    pub cliente_id: i32,
}
#[derive(Debug,FromRow,Serialize,Deserialize)]
pub struct ContratoTemplateDto {
    pub nome: String,
    pub data: String
}

#[derive(Serialize,Deserialize,Debug,FromRow)]
pub struct ContratoTemplate {
    pub id: i32,
    pub nome: String,
    pub path: String,
}
