use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use askama::Template;
use axum_extra::extract::Form;
use cnpj::Cnpj;
use cpf::Cpf;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};
use sqlx::{prelude::{FromRow, Type}, query, query_as, Decode, Encode, PgPool, Postgres};

use super::{mikrotik::Mikrotik, planos::Plano};
pub async fn show_cliente_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let clients = query_as!(Cliente, "SELECT * FROM clientes")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch clients: {:?}", e);
            Html("<p>Failed to fetch clients</p>".to_string())
        }).expect("Failed to fetch clients");

    let template = ClienteListTemplate { clients };

    let template = template.render().map_err(|e| -> _ {
        error!("Failed to render client list template: {:?}", e);
        Html("<p>Failed to render client list template</p>".to_string())
    }).expect("Failed to render client list template");

    Html(template)
}
// Add the handler for showing the edit form
pub async fn show_cliente_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let client = query_as!(Cliente, "SELECT * FROM clientes WHERE id = $1", id)
        .fetch_one(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch client: {:?}", e);
            Html("<p>Failed to fetch client</p>".to_string())
        }).expect("Failed to fetch client");

    let mikrotik_list = query_as!(Mikrotik, "SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Mikrotik: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        }).expect("Failed to fetch Mikrotik");

    let plan_list = query_as!(Plano, "SELECT * FROM planos")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Planos: {:?}", e);
            Html("<p>Failed to fetch Planos</p>".to_string())
        }).expect("Failed to fetch Planos");

    let template = ClienteEditTemplate {
        client,
        mikrotik_options: mikrotik_list,
        plan_options: plan_list,
    };

    let template = template.render().map_err(|e| -> _ {
        error!("Failed to render client edit template: {:?}", e);
        Html("<p>Failed to render client edit template</p>".to_string())
    }).expect("Failed to render client edit template");

    Html(template)
}

pub async fn delete_cliente(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    query!("DELETE FROM clientes WHERE id = $1", id)
        .execute(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to delete client: {:?}", e);
            Html("<p>Failed to delete client</p>".to_string())
        }).expect("Failed to delete client");

    Redirect::to("/clientes").into_response()
}

// Add the handler for updating the client
pub async fn update_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(client): Form<Cliente>,
) -> impl IntoResponse {
    query!(
        "UPDATE clientes SET tipo = $1, nome = $2, email = $3, cpf_cnpj = $4, formatted_cpf_cnpj = $5, telefone = $6, login = $7, senha = $8, endereco_id = $9, mikrotik_id = $10, plano_id = $11 WHERE id = $12",
        client.tipo,
        client.nome,
        client.email,
        client.cpf_cnpj,
        client.formatted_cpf_cnpj,
        client.telefone,
        client.login,
        client.senha,
        //mikrotik endereco e plano
        //sao obrigatorios para criar o cliente
        client.endereco_id,
        client.mikrotik_id,
        client.plano_id,
        client.id
    )
    .execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to update client: {:?}", e);
        Html("<p>Failed to update client</p>".to_string())
    }).expect("Failed to update client");

    Redirect::to("/clientes").into_response()
}
pub async fn show_cliente_form(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let mikrotik_list = query_as!(Mikrotik,"SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ { 
            debug!("Failed to fetch Mikrotik: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        }).expect("error fetching mikrotik");

    let plan_list  = query_as!(Plano,"SELECT * FROM planos")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            debug!("Failed to fetch Planos: {:?}", e);
            Html("<p>Failed to fetch Planos</p>".to_string())
        }).expect("Failed to fetch Planos");

    let template = ClienteFormTemplate {
        mikrotik_options: mikrotik_list,
        plan_options: plan_list,
    };

    let template = template.render().map_err(|e| -> _ {
        error!("Failed to render cliente form template: {:?}", e);
        Html("<p>Failed to render cliente form template</p>".to_string())
    }).expect("Failed to render cliente form template");

    Html(template)
}

pub async fn register_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mut client): Form<ClienteDto>,
) -> impl IntoResponse {

    //this should be done on the frontend
    //so the parse and the valid may be no ops
    //lets make the frontend do this with the htmx when the client inputs data
    //would just need to have a formatted and unformated field
    match client.tipo {
        TipoPessoa::PessoaFisica => {
            if cpf::valid(&client.cpf_cnpj) {
                client.formatted_cpf_cnpj = client.cpf_cnpj.parse::<Cpf>().map_err(|e| -> _ {
                    error!("Failed to parse cpf/cnpj: {:?}", e);
                    Html("<p>Failed to parse cpf/cnpj</p>".to_string())
                }).expect("Failed to parse cpf/cnpj").to_string(); 
            } else {
                return Html("<p>Invalid CPF</p>".to_string()).into_response();
            }
        }
        TipoPessoa::PessoaJuridica => {
            if cnpj::valid(&client.cpf_cnpj) {
                client.formatted_cpf_cnpj = client.cpf_cnpj.parse::<Cnpj>().map_err(|e| -> _ {
                    error!("Failed to parse cpf/cnpj: {:?}", e);
                    Html("<p>Failed to parse cpf/cnpj</p>".to_string())
                }).expect("Failed to parse cpf/cnpj").to_string(); 
            } else {
                return Html("<p>Invalid CNPJ</p>".to_string()).into_response();
            }
        }
    }

    save_to_db(pool, &client).await.map_err(|e| -> _ {
        error!("Failed to save client to db: {:?}", e);
        Html("<p>Failed to save client to db</p>".to_string())
    }).expect("Failed to save client to db");

    Redirect::to("/clientes").into_response()
}


//pass the sqlx logic for saving to the db to here
pub async fn save_to_db(pool: Arc<PgPool>, client: &ClienteDto) -> Result<(), anyhow::Error> {
    let endereco = EnderecoDto {
        cep: client.endereco.cep.cep.clone(),
        rua: client.endereco.rua.clone(),
        numero: client.endereco.numero.clone(),
        bairro: client.endereco.bairro.clone(),
        cidade: client.endereco.cidade.clone(),
        estado: client.endereco.estado.clone(),
        complemento: client.endereco.complemento.clone(),
        ibge: client.endereco.ibge.clone(),
    }; 

    let endereco_id  = query!(
        "INSERT INTO enderecos (cep, rua, numero, bairro, complemento, cidade, estado, ibge_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        endereco.cep,
        endereco.rua,
        endereco.numero,
        endereco.bairro,
        endereco.complemento,
        endereco.cidade,
        endereco.estado,
        endereco.ibge
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| -> _ {
        error!("Failed to insert endereco: {:?}", e);
        anyhow::anyhow!("Failed to insert endereco {e}")
    }).expect("Failed to insert endereco");

    query!(
        "INSERT INTO clientes (tipo, nome, email, cpf_cnpj, formatted_cpf_cnpj, telefone, login, senha, endereco_id, mikrotik_id, plano_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        client.tipo.as_bool(),
        client.nome,
        client.email,
        client.cpf_cnpj,
        client.formatted_cpf_cnpj,
        client.telefone,
        client.login,
        client.senha,
        endereco_id.id,
        client.mikrotik_id,
        client.plan_id
    )
    .execute(&*pool)
    .await
    .map_err(|e| -> _ {
        error!("Failed to insert client: {:?}", e);
        anyhow::anyhow!("Failed to insert client {e}")
    }).expect("Failed to insert client");

    Ok(())
}

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

#[derive(Template)]
#[template(path = "cliente_edit.html")]
struct ClienteEditTemplate {
    client: Cliente,
    mikrotik_options: Vec<Mikrotik>,
    plan_options: Vec<Plano>,
}

impl ClienteEditTemplate {
    fn is_pessoa_fisica(&self) -> bool {
        !self.client.tipo
    }

    fn is_pessoa_juridica(&self) -> bool {
        self.client.tipo
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq,Clone, Copy)]
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
#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct ClienteDto {
    //TODO: maybe change this to a enum
    //0 for pf and 1 for pj
    #[serde(flatten)]
    pub tipo: TipoPessoa,
    pub nome: String,
    pub email: String,
    //should do a validation here
    //if its pj should be cnpj validation and pf cpf validation
    pub cpf_cnpj: String,
    pub formatted_cpf_cnpj: String,
    #[serde(flatten)]
    pub endereco: Endereco,
    pub telefone: String,
    pub login: String,
    pub senha: String,
    pub mikrotik_id: Option<i32>,
    pub plan_id: i32,
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
    pub endereco_id: Option<i32>,
    pub telefone: String,
    pub login: Option<String>,
    pub senha: Option<String>,
    pub mikrotik_id: Option<i32>,
    pub plano_id: Option<i32>, 
//    pub contrato_id: Option<Vec<i32>>
}

//TODO: add display trait to endereco
#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Endereco {
    pub id: i32,
    pub cep: Cep,
    pub rua: String,
    pub numero: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub ibge: String,
}

impl Display for Endereco {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {} {} {} {}", self.rua, self.numero.as_deref().unwrap_or(""), self.bairro, self.cidade, self.estado, self.complemento.as_deref().unwrap_or(""), self.ibge)
    }
}

#[derive(Debug)]
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