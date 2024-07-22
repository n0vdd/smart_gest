use std::sync::Arc;

use askama::Template;
use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
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

pub async fn find_plano_by_cliente(pool:&PgPool,cliente_id: i32) -> Result<Plano,sqlx::Error> {
    query_as!(
        Plano,
        "SELECT planos.* FROM planos INNER JOIN clientes ON planos.id = clientes.plano_id WHERE clientes.id = $1",
        cliente_id
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_plano(Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<i32>)
    -> impl IntoResponse {
    query!(
        "DELETE FROM planos WHERE id = $1",
        id 
    ).execute(&*pool).await.map_err(
        |e| {
            error!("Failed to delete plano: {:?}", e);
            e
        }
    ).expect("Erro ao deletar plano");

    Redirect::to("/plano")
}

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
    
    let template = PlanosListTemplate { planos };
    let html = template.render().map_err(|e| {
        error!("Failed to render planos list template: {:?}", e);
        e
    }).expect("Erro ao renderizar planos list template");

    Html(html)
}

pub async fn update_plano(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<i32>,
    Form(plano): Form<PlanoDto>,
) -> impl IntoResponse  {
    query!(
        "UPDATE planos SET nome = $1, valor = $2, velocidade_up = $3, velocidade_down = $4, descricao = $5, contrato_template_id = $6 WHERE id = $7",
        plano.nome,
        plano.valor,
        plano.velocidade_up,
        plano.velocidade_down,
        plano.descricao,
        plano.contrato_template_id,
        id
    )
    .execute(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to update plano: {:?}", e);
        e
    }).expect("Erro ao atualizar plano");

    Redirect::to("/plano")
}

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
        e
    }).expect("Erro ao buscar plano");

    let contracts = query_as!(ContratoTemplate, "SELECT * FROM contratos_templates")
        .fetch_all(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch contract templates: {:?}", e);
            e
        }).expect("Erro ao buscar contratos");
    
    let template = PlanoEditFormTemplate { plano, contracts };
    let html = template.render().map_err(|e| {
        error!("Failed to render plano edit form template: {:?}", e);
        e
    }).expect("Erro ao renderizar plano edit form template");

    Html(html)
}


pub async fn show_planos_form(Extension(pool): Extension<Arc<PgPool>>) -> Html<String> {
    let contracts= query_as!(ContratoTemplate, "SELECT * FROM contratos_templates")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch contract templates: {:?}", e);
            Html("Failed to fetch contract templates".to_string())
        }).expect("Failed to fetch contract templates");

    let template = PlanosFormTemplate {
        contracts,
    };

    let template = template.render().map_err(|e| -> _ {
        error!("Failed to render planos form template: {:?}", e);
        Html("Failed to render planos form template".to_string())
    }).expect("Failed to render planos form template");

    Html(template)
}

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
        return Html("Failed to insert Plano".to_string());
    }).expect("Failed to insert Plano");

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
    pub descricao: Option<String>,
    // TODO vincular o template de contrato de acordo com o plano
    //Tenho que representar os contratos na db
    //pub contrato: Option<String>,
    pub contrato_template_id: i32,
    pub created_at : Option<PrimitiveDateTime>,
    pub updated_at : Option<PrimitiveDateTime>
}

