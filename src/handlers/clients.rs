use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use time::{Date, PrimitiveDateTime};
use tracing::{debug, error};
use validator::Validate;
use std::fmt;
use askama::Template;
use axum_extra::extract::Form;
use cnpj::Cnpj;
use cpf::Cpf;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::{prelude::{FromRow, Type}, query, query_as, Decode, Encode, PgPool, Postgres};

use crate::handlers::{mikrotik::Mikrotik, planos::Plano};

// Structs and Enums
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
    pub mikrotik_id: Option<i32>,
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
    //The only 2 i32
    //they are commented out in the html now
    pub mikrotik_id: Option<i32>,
    pub plano_id: Option<i32>,
    // pub contrato_id: Option<Vec<i32>>
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TipoPessoa {
    PessoaFisica,
    PessoaJuridica,
}

impl TipoPessoa {
    fn as_bool(&self) -> bool {
        match self {
            TipoPessoa::PessoaFisica => false,
            TipoPessoa::PessoaJuridica => true,
        }
    }

    fn from_bool(value: bool) -> Self {
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

// Handlers

pub async fn show_cliente_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let clients = query_as!(Cliente, "SELECT * FROM clientes")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch clients: {:?}", e);
            Html("<p>Failed to fetch clients</p>".to_string())
        })
        .expect("Failed to fetch clients");

    let template = ClienteListTemplate { clients }
        .render()
        .map_err(|e| -> _ {
            error!("Failed to render client list template: {:?}", e);
            Html("<p>Failed to render client list template</p>".to_string())
        })
        .expect("Failed to render client list template");

    Html(template)
}

/* TODO deal with edit form later
//lets look what the rest of the things we have to do
//need to configure radius and the importante shit
pub async fn show_cliente_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {

    let client = query_as!(Cliente, "SELECT * FROM clientes WHERE id = $1", id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch client: {:?}", e);
            Html("<p>Failed to fetch client</p>".to_string())
        })
        .expect("Failed to fetch client");

    let mikrotik_list = query_as!(Mikrotik, "SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch Mikrotik: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        })
        .expect("Failed to fetch Mikrotik");

    let plan_list = query_as!(Plano, "SELECT * FROM planos")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch Planos: {:?}", e);
            Html("<p>Failed to fetch Planos</p>".to_string())
        })
        .expect("Failed to fetch Planos");

    let template = ClienteEditTemplate {
        &client,
        mikrotik_options: mikrotik_list,
        plan_options: plan_list,
    }
    .render()
    .map_err(|e| -> _ {
        error!("Failed to render client edit template: {:?}", e);
        Html("<p>Failed to render client edit template</p>".to_string())
    })
    .expect("Failed to render client edit template");

    Html(template)
}
*/

pub async fn delete_cliente(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {

    query!("DELETE FROM clientes WHERE id = $1", id)
        .execute(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to delete client: {:?}", e);
            Html("<p>Failed to delete client</p>".to_string())
        })
        .expect("Failed to delete client");

    Redirect::to("/cliente").into_response()
}

pub async fn update_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(client): Form<Cliente>,
) -> impl IntoResponse {


    query!(
        "UPDATE clientes SET tipo = $1, nome = $2, email = $3, cpf_cnpj = $4, formatted_cpf_cnpj = $5,
        telefone = $6, login = $7, senha = $8, cep = $9, rua = $10, numero = $11, bairro = $12,
        complemento = $13, cidade = $14, estado = $15, ibge_code = $16, mikrotik_id = $17,
        plano_id = $18 WHERE id = $19",
        client.tipo,
        client.nome,
        client.email,
        client.cpf_cnpj,
        client.formatted_cpf_cnpj,
        client.telefone,
        client.login,
        client.senha,
        client.cep,
        client.rua,
        client.numero,
        client.bairro,
        client.complemento,
        client.cidade,
        client.estado,
        client.ibge_code,
        client.mikrotik_id,
        client.plano_id,
        client.id
    )
    .execute(&*pool)
    .await
    .map_err(|e| -> _ {
        error!("Failed to update client: {:?}", e);
        Html("<p>Failed to update client</p>".to_string())
    })
    .expect("Failed to update client");

    Redirect::to("/cliente").into_response()
}

pub async fn show_cliente_form(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {

    let mikrotik_list = query_as!(Mikrotik, "SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            debug!("Failed to fetch Mikrotik: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        })
        .expect("error fetching mikrotik");

    let plan_list = query_as!(Plano, "SELECT * FROM planos")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            debug!("Failed to fetch Planos: {:?}", e);
            Html("<p>Failed to fetch Planos</p>".to_string())
        })
        .expect("Failed to fetch Planos");

    let template = ClienteFormTemplate {
        mikrotik_options: mikrotik_list,
        plan_options: plan_list,
    }.render()
    .map_err(|e| -> _ {
        error!("Failed to render cliente form template: {:?}", e);
        Html("<p>Failed to render cliente form template</p>".to_string())
    })
    .expect("Failed to render cliente form template");

    Html(template)
}

//TODO deal with the validation of the cpf/cnpj on the frontend
pub async fn register_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mut client): Form<ClienteDto>,
) -> impl IntoResponse {
    // Validation for CPF/CNPJ
    match client.tipo {
        TipoPessoa::PessoaFisica => {
            if cpf::valid(&client.cpf_cnpj) {
                client.formatted_cpf_cnpj = client
                    .cpf_cnpj
                    .parse::<Cpf>()
                    .map_err(|e| -> _ {
                        error!("Failed to parse cpf/cnpj: {:?}", e);
                        Html("<p>Failed to parse cpf/cnpj</p>".to_string())
                    })
                    .expect("Failed to parse cpf/cnpj")
                    .to_string();
            } else {
                return Html("<p>Invalid CPF</p>".to_string()).into_response();
            }
        },
        TipoPessoa::PessoaJuridica => {
            if cnpj::valid(&client.cpf_cnpj) {
                client.formatted_cpf_cnpj = client
                    .cpf_cnpj
                    .parse::<Cnpj>()
                    .map_err(|e| -> _ {
                        error!("Failed to parse cpf/cnpj: {:?}", e);
                        Html("<p>Failed to parse cpf/cnpj</p>".to_string())
                    })
                    .expect("Failed to parse cpf/cnpj")
                    .to_string();
            } else {
                return Html("<p>Invalid CNPJ</p>".to_string()).into_response();
            }
        }
    }

    query!(
        "INSERT INTO clientes (
            tipo, nome, email, cpf_cnpj, formatted_cpf_cnpj, telefone, login, senha, 
            mikrotik_id, plano_id, cep, rua, numero, bairro, complemento, cidade, estado, ibge_code
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        )",
        client.tipo.as_bool(),
        client.nome,
        client.email,
        client.cpf_cnpj,
        client.formatted_cpf_cnpj,
        client.telefone,
        client.login,
        client.senha,
        client.mikrotik_id,
        client.plano_id,
        client.endereco.cep,
        client.endereco.rua,
        client.endereco.numero,
        client.endereco.bairro,
        client.endereco.complemento,
        client.endereco.cidade,
        client.endereco.estado,
        client.endereco.ibge
    )
    .execute(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to insert client: {:?}", e);
        anyhow::anyhow!("Failed to insert client {e}")
    })
    .expect("Failed to insert client");

    Redirect::to("/cliente").into_response()
}

// Templates

#[derive(Template)]
#[template(path = "cliente_add.html")]
struct ClienteFormTemplate {
    mikrotik_options: Vec<Mikrotik>,
    plan_options: Vec<Plano>,
}

#[derive(Template)]
#[template(path = "cliente_list.html")]
struct ClienteListTemplate {
    clients: Vec<Cliente>,
}

/* 
#[derive(Template)]
#[template(path = "cliente_edit.html")]
struct ClienteEditTemplate<'a> {
    client: &'a Cliente,
    mikrotik_options: Vec<Mikrotik>,
    plan_options: Vec<Plano>,
}

impl ClienteEditTemplate<'_> {
    fn is_pessoa_fisica(&self) -> bool {
        !self.client.tipo
    }

    fn is_pessoa_juridica(&self) -> bool {
        self.client.tipo
    }
}
*/

impl fmt::Display for Endereco {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}, {}, {}, {}, {}", self.rua, self.numero.clone().unwrap_or_default(),
        self.bairro, self.cidade, self.estado, self.cep.cep)
    }
}

pub async fn fetch_tipo_clientes_before_date(
    pool: &PgPool,
    date: PrimitiveDateTime
) -> Vec<TipoPessoa> {
    query!("SELECT tipo FROM clientes WHERE created_at < $1", date)
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch clients: {:?}", e);
            e
        })
        .expect("Failed to fetch clients")
        .iter().map(|row| {
            TipoPessoa::from_bool(row.tipo)
        }).collect()
}