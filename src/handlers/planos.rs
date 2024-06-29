use std::sync::Arc;

use askama::Template;
use axum::{response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, PgPool};

pub async fn show_planos_form() -> Html<String> {
    let template = PlanosFormTemplate;

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
        "INSERT INTO planos (nome, valor, velocidade_up,velocidade_down, descricao, tecnologia)
        VALUES ($1, $2, $3, $4, $5, $6)",
        plano.nome,
        plano.valor,
        plano.velocidade_up,
        plano.velocidade_down,
        plano.descricao,
        plano.tecnologia) 
    .execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to insert Plano: {:?}", e);
        return Html("Failed to insert Plano".to_string());
    }).expect("Failed to insert Plano");

    Redirect::to("/planos")
}
//need to show planos_form and register plano to db
#[derive(Template)]
#[template(path = "planos_add.html")]
struct PlanosFormTemplate;

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct PlanoDto {
    pub nome: String,
    pub valor: f32,
    pub velocidade_up: i32,
    pub velocidade_down: i32,
    pub descricao: Option<String>,
    pub tecnologia: Option<String>,
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
    pub tecnologia: Option<String>,

}

