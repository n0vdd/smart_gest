use std::sync::Arc;

use askama::Template;
use axum::{response::Html, Extension};
use axum_extra::extract::Form;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

pub async fn show_planos_form() -> Html<String> {
    let template = PlanosFormTemplate;
    Html(template.render().expect("Failed to render planos form template"))
}

pub async fn register_plano(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(plano): Form<PlanoDto>,
) -> Html<String> {
    sqlx::query(
        "INSERT INTO planos (name, valor, velocidade_up,velocidade_down, descricao, tecnologia)
        VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&plano.nome)
    .bind(&plano.valor)
    .bind(&plano.velocidade_up)
    .bind(&plano.velocidade_down)
    .bind(&plano.descricao)
    .bind(&plano.tecnologia)
    .execute(&*pool)
    .await
    .expect("Failed to insert Plano");

    Html(format!("<p>Plano {} registered successfully!</p>", plano.nome))
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
    // poderia ser uma enum
    // TODO o template do contrato na real pode ser vinculado ao plano
    pub tecnologia: Option<String>,
}

