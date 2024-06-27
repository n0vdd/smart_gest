use askama::Template;
use axum::{response::Html, Extension};
use axum_extra::extract::Form;
use log::debug;
use serde::Deserialize;
use std::{net::IpAddr, sync::Arc};
use sqlx::{prelude::FromRow, query, PgPool};

#[derive(Template)]
#[template(path = "mikrotik_add.html")]
struct MikrotikFormTemplate;

pub async fn show_mikrotik_form() -> Html<String> {
    let template = MikrotikFormTemplate;
    Html(template.render().unwrap())
}


#[derive(Deserialize , Debug, FromRow)]
pub struct MikrotikDto {
    pub name: String,
    pub ip: IpAddr,
    pub secret: String,
    pub max_clientes: Option<i32>,
    pub user: Option<String>,
    pub password: Option<String>,

}

pub async fn register_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<MikrotikDto>,
) -> Html<String> {
    if !mikrotik.ip.is_ipv4() {
        return Html("<p>Invalid IP</p>".to_string());
    }
    debug!("mikrotik:{:?}",mikrotik);

    //how do i make this check at compile time?
    sqlx::query(
        "INSERT INTO mikrotik (name, ip, secret, max_clients, ssh_login, ssh_password)
        VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&mikrotik.name)
    .bind(&mikrotik.ip.to_string())
    .bind(&mikrotik.secret)
    .bind(&mikrotik.max_clientes)
    .bind(&mikrotik.user)
    .bind(&mikrotik.password)
    .execute(&*pool)
    .await
    .expect("Failed to insert Mikrotik");


    Html(format!("<p>Mikrotik {} registered successfully!</p>", mikrotik.name))
}

//TODO funcao que retorna todos os mikrotiks
//serao duas na verdade, uma para listagem com todos os dados
//e outra para o dropdown com a id e nome(ver como isso sera feito em conjuncacao com o frontend)
//askama sera util aqui