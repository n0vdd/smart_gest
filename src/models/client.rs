use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, FromRow, Postgres, Type};
use time::PrimitiveDateTime;
use validator::Validate;

use super::endereco::EnderecoDto;

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
    pub telefone: String,
    pub login: String,
    pub senha: String,
    pub mikrotik_id: i32,
    pub plano_id: i32,
    pub contrato_id: Option<Vec<i32>>
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Cliente {
    pub id: i32,
    pub tipo: bool,
    pub nome: String,
    pub email: String,
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
    //TODO convert this on cliente_edit.html
    pub login: Option<String>,
    //TODO convert this on cliente_edit.html
    pub senha: Option<String>,
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