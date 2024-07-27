use std::{str::FromStr, sync::Arc};

use askama::Template;
use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use radius::{create_radius_plano, PlanoRadiusDto};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, PgPool};
use time::PrimitiveDateTime;
use tracing::error;

use super::contrato::ContratoTemplate;

#[derive(Template)]
#[template(path = "plano_list.html")]
struct PlanosListTemplate {
    planos: Vec<Plano>,
}

#[derive(Template)]
#[template(path = "plano_edit.html")]
struct PlanoEditFormTemplate {
    plano: Plano,
    contracts: Vec<ContratoTemplate>,
}


//Recebe a id de um cliente
//Utiliza a id do cliente para achar qual o plano associado aquele cliente
//retorna o plano
pub async fn find_plano_by_cliente(pool:&PgPool,cliente_id: i32) -> Result<Plano,anyhow::Error> {
    query_as!(
        Plano,
        "SELECT planos.* FROM planos INNER JOIN clientes ON planos.id = clientes.plano_id WHERE clientes.id = $1",
        cliente_id
    )
    .fetch_one(pool)
    .await.map_err(|e| {
        error!("Failed to fetch plano: {:?}", e);
        anyhow::anyhow!("Failed to fetch plano data related to the cliente {cliente_id} from db")
    })
}

//Deleta o plano basead na id passada(pelo button de delete)
pub async fn delete_plano(Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<i32>)
    -> impl IntoResponse {
    query!(
        "DELETE FROM planos WHERE id = $1",
        id 
    ).execute(&*pool).await.map_err(|e| {
        error!("Failed to delete plano: {:?}", e);
        e
    }).expect("Erro ao deletar plano");

    Redirect::to("/plano")
}

//Pega todos os planos criados na db
//popula uma template com eles
//retorna a listagem(template)
pub async fn list_planos(Extension(pool): Extension<Arc<PgPool>>) -> Html<String> {
    let planos = query_as!(
        Plano,
        "SELECT * FROM planos"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch planos: {:?}", e);
        e
    }).expect("Erro ao buscar planos");
    
    let template = PlanosListTemplate { planos }.render().map_err(|e| {
        error!("Failed to render planos list template: {:?}", e);
        e
    }).expect("Erro ao renderizar planos list template");

    Html(template)
}

//Recebe os dados de edicao de um plano
//Atualiza o plano na db
//Retorna a listagem com todos os planos
pub async fn update_plano(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(plano): Form<Plano>,
) -> impl IntoResponse  {
    query!(
        "UPDATE planos SET nome = $1, valor = $2, velocidade_up = $3, velocidade_down = $4, descricao = $5, contrato_template_id = $6 WHERE id = $7",
        plano.nome,
        plano.valor,
        plano.velocidade_up,
        plano.velocidade_down,
        plano.descricao,
        plano.contrato_template_id,
        plano.id
    )
    .execute(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to update plano: {:?}", e);
        e
    }).expect("Erro ao atualizar plano");

    Redirect::to("/plano")
}

//Recebe a id de um plano pelo button de editar
//Busca o plano na db
//Busca todas as templates de contrato na db(popular edit form tambem)
//Popula uma template com os dados do plano
//Exibe o formulario de exibicao populado para o usuario
pub async fn show_plano_edit_form(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<i32>
) -> Html<String> {
    let plano = query_as!(
        Plano,
        "SELECT * FROM planos WHERE id = $1",
        id
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch plano: {:?}", e);
        return Html("<p>Failed to fetch plano for editing</p>".to_string())
    }).expect("Erro ao buscar plano");

    //Possivel selecionar qualquer uma das templates de contrato para ser usado pelo plano
    //Elas sao usadas ao criar um contrato para o cliente
    let contracts = query_as!(ContratoTemplate, "SELECT * FROM contratos_templates")
        .fetch_all(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch contract templates: {:?}", e);
            return Html("<p>Failed to fetch contract templates</p>".to_string())
        }).expect("Erro ao buscar contratos");
    
    let template = PlanoEditFormTemplate { plano, contracts }.render().map_err(|e| {
        error!("Failed to render plano edit form template: {:?}", e);
        return Html("<p>Failed to render plano edit form template</p>".to_string())
    }).expect("Erro ao renderizar plano edit form template");

    Html(template)
}


//Acha todas as templates de contrato na db
//Popula a template de criacao de planos com as opcoes de contrato
//renderiza a template e a retorna para o usuario
pub async fn show_planos_form(Extension(pool): Extension<Arc<PgPool>>) -> Html<String> {
    //Usados para gerar o contrato do cliente posteriormente
    let contracts= query_as!(ContratoTemplate, "SELECT * FROM contratos_templates")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch contract templates: {:?}", e);
            return Html("<p>Failed to fetch contract templates</p>".to_string())
        }).expect("Failed to fetch contract templates");

    let template = PlanosFormTemplate {
        contracts,
    }.render().map_err(|e| -> _ {
        error!("Failed to render planos form template: {:?}", e);
        return Html("<p>Failed to render planos form template</p>".to_string())
    }).expect("Failed to render planos form template");

    Html(template)
}


//Recebe os dados de um plano
//Salva o mesmo para a db
//Retorna a listagem de planos
pub async fn register_plano(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(plano): Form<PlanoDto>,
) -> impl IntoResponse {
    query!(
        "INSERT INTO planos (nome, valor, velocidade_up,velocidade_down, descricao, contrato_template_id)
        VALUES ($1, $2, $3, $4, $5, $6)",
        plano.nome,
        plano.valor,
        plano.velocidade_up,
        plano.velocidade_down,
        plano.descricao,
        //plano.tecnologia,
        plano.contrato_template_id) 
    .execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to insert Plano: {:?}", e);
        return Html("<p>Failed to insert Plano</p>".to_string());
    }).expect("Failed to insert Plano");


    let radius_plano = PlanoRadiusDto {
        nome: plano.nome,
        velocidade_up: plano.velocidade_up,
        velocidade_down: plano.velocidade_down
    };

    create_radius_plano(radius_plano).await.map_err(|e| {
        error!("Failed to create radius plano: {:?}", e);
        return Html("<p>Failed to create radius plano</p>".to_string());
    }).expect("Failed to create radius plano");

    Redirect::to("/plano")
}
//need to show planos_form and register plano to db
#[derive(Template)]
#[template(path = "planos_add.html")]
struct PlanosFormTemplate {
    contracts: Vec<ContratoTemplate>,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct PlanoDto {
    pub nome: String,
    pub valor: f32,
    pub velocidade_up: i32,
    pub velocidade_down: i32,
    pub descricao: Option<String>,
    pub contrato_template_id: i32
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Plano {
    pub id: i32,
    pub nome: String,
    pub valor: f32,
    pub velocidade_up: i32,
    pub velocidade_down: i32,
    pub tipo_pagamento: TipoPagamento,
    pub descricao: Option<String>,
    pub contrato_template_id: i32,
    pub created_at : Option<PrimitiveDateTime>,
    pub updated_at : Option<PrimitiveDateTime>
}


#[derive(Deserialize, Serialize, Debug,Clone)]
pub enum TipoPagamento {
    Boleto,
    Pix,
    CartaoCredito,
}

impl FromStr for TipoPagamento {
    type Err = ();

    fn from_str(input: &str) -> Result<TipoPagamento, Self::Err> {
        match input {
            "BOLETO" => Ok(TipoPagamento::Boleto),
            "PIX" => Ok(TipoPagamento::Pix),
            "CREDIT_CARD" => Ok(TipoPagamento::CartaoCredito),
            _ => Err(()),
        }
    }
}

impl From<String> for TipoPagamento {
    fn from(s: String) -> TipoPagamento {
        TipoPagamento::from_str(&s).unwrap_or(TipoPagamento::Boleto) // default to Boleto or handle error appropriately
    }
}

impl ToString for TipoPagamento {
    fn to_string(&self) -> String {
        match self {
            TipoPagamento::Boleto => "BOLETO".to_string(),
            TipoPagamento::Pix => "PIX".to_string(),
            TipoPagamento::CartaoCredito => "CREDIT_CARD".to_string(),
        }
    }
}

// Implementing Into<String> for TipoPagamento
impl Into<String> for TipoPagamento {
    fn into(self) -> String {
        self.to_string()
    }
}