use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, FromRow, Postgres, Type};
use time::PrimitiveDateTime;
use validator::Validate;

use crate::config::endereco_model::EnderecoDto;


#[derive(Deserialize, Serialize, Debug, FromRow,Validate)]
pub struct ClienteDto {
    pub tipo: TipoPessoa,
    pub nome: String,
    #[validate(email)]
    pub email: String,
    pub cpf_cnpj: String,
    pub formatted_cpf_cnpj: String,
    #[serde(flatten)]
    pub endereco: EnderecoDto,
    pub gera_dici: bool,
    pub gera_nf: bool,
    pub add_to_asaas: bool,
    pub telefone: String,
    pub login: String,
    pub senha: String,
    pub mikrotik_id: i32,
    pub plano_id: i32,
    pub contrato_id: Option<Vec<i32>>
}

pub struct ClienteNf {
    pub nome: String,
    pub email: String,
    pub cpf_cnpj: String,
    pub gera_nf: bool,
    pub rua: String,
    pub numero: Option<String>,
    //pub bairro: String,
    //pub cidade: String,
    //pub estado: String,
    pub complemento: Option<String>,
    pub cep: String,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Cliente {
    pub id: i32,
    pub tipo: bool,
    pub nome: String,
    pub email: String,
    //TODO i have to guarantee this is not formated
    //the cliente can pass an already formatted cpf/cnpj and i would have to strip it off
    pub cpf_cnpj: String,
    pub formatted_cpf_cnpj: String,
    pub cep: String,
    pub rua: String,
    //TODO convert this on cliente_edit.html
    pub numero: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    //TODO convert this on cliente_edit.html
    pub complemento: Option<String>,
    pub ibge_code: String,
    pub telefone: String,
    pub gera_dici: bool,
    pub gera_nf: bool,
    pub add_to_asaas: bool,
    //TODO convert this on cliente_edit.html
    pub login: String,
    //TODO convert this on cliente_edit.html
    pub senha: String,
    pub mikrotik_id: i32,
    pub plano_id: i32,
    // pub contrato_id: Option<Vec<i32>>
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TipoPessoa {
    PessoaFisica,
    PessoaJuridica,
}

impl TipoPessoa {
    pub fn as_bool(&self) -> bool {
        match self {
            TipoPessoa::PessoaFisica => false,
            TipoPessoa::PessoaJuridica => true,
        }
    }

    pub fn from_bool(value: bool) -> Self {
        match value {
            false => TipoPessoa::PessoaFisica,
            true => TipoPessoa::PessoaJuridica,
        }
    }
}

impl Type<Postgres> for TipoPessoa {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <bool as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for TipoPessoa {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <bool as Encode<Postgres>>::encode(self.as_bool(), buf)
    }
}

impl<'r> Decode<'r, Postgres> for TipoPessoa {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let int_value = <bool as Decode<Postgres>>::decode(value)?;
        Ok(TipoPessoa::from_bool(int_value))
    }
}

pub struct SimpleCliente {
    pub id: i32,
    pub login: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientDataJson {
    pub uuid: String,
    pub id: String,
    pub nome: String,
    pub email: Option<String>,
    pub celular: Option<String>,
    pub login: String,
    pub senha: String,
    pub endereco: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
    pub complemento: Option<String>,
    pub cidade: Option<String>,
    pub cep: Option<String>,
    pub estado: Option<String>,
    pub cpf_cnpj: String,
}

/// Struct for the list of clients received from the API.
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientList {
    pub clientes: Vec<ClientDataJson>,
}