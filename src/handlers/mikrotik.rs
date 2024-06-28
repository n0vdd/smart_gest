use askama::Template;
use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, str::FromStr, sync::Arc};
use sqlx::{prelude::FromRow, query, query_as, PgPool};

pub async fn show_mikrotik_form() -> Html<String> {
    let template = MikrotikFormTemplate;
    Html(template.render().expect("Failed to render Mikrotik form template"))
}

pub async fn register_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<MikrotikDto>,
) -> impl IntoResponse {

    if mikrotik.ip.is_loopback() || mikrotik.ip.is_unspecified() {
        return Html("<p>Invalid IP</p>".to_string()).into_response();
    }
    debug!("mikrotik:{:?}",mikrotik);

    //how do i make this check at compile time?
    query!(
        "INSERT INTO mikrotik (nome, ip, secret, max_clientes, ssh_login, ssh_password)
        VALUES ($1, $2, $3, $4, $5, $6)",
        mikrotik.nome,
        mikrotik.ip.to_string(),
        mikrotik.secret,
        mikrotik.max_clientes,
        mikrotik.login,
        mikrotik.senha)
    .execute(&*pool)
    .await
    .map_err(|e| 
        debug!("Failed to insert Mikrotik: {:?}", e)
    ).expect("Failed to insert Mikrotik");

    Redirect::to("/mikrotik").into_response()
}

pub async fn show_mikrotik_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let mikrotik_list  = query_as!(Mikrotik,"SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await.map_err(|e| 
            debug!("Failed to fetch Mikrotik: {:?}", e)
        ).expect("Failed to fetch Mikrotik");

    let template = MikrotikListTemplate {
        mikrotik_options: mikrotik_list,
    };

    Html(template.render().expect("Failed to render Mikrotik list template"))
}

pub async fn show_mikrotik_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let mikrotik = query_as!(Mikrotik,"SELECT * FROM mikrotik WHERE id = $1",id)
        .fetch_one(&*pool)
        .await
        .expect("Failed to fetch Mikrotik");

    let template = MikrotikEditTemplate {
        mikrotik,
    };

    Html(template.render().expect("Failed to render Mikrotik edit template"))
}

pub async fn update_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<Mikrotik>,
) -> impl IntoResponse {
    let ip = Ipv4Addr::from_str(&mikrotik.ip).expect("Failed to parse IP");
    if ip.is_loopback() || ip.is_unspecified() {
        return Html("<p>Invalid IP</p>".to_string()).into_response();
    }

    query!(
        "UPDATE mikrotik SET nome = $1, ip = $2, secret = $3, max_clientes = $4, ssh_login = $5, ssh_password = $6 WHERE id = $7",
        mikrotik.nome,
        mikrotik.ip,
        mikrotik.secret,
        mikrotik.max_clientes,
        mikrotik.ssh_login,
        mikrotik.ssh_password,
        mikrotik.id
    )
    .execute(&*pool)
    .await
    .expect("Failed to update Mikrotik");

    Redirect::to("/mikrotik").into_response()
}

#[derive(Template)]
#[template(path = "mikrotik_edit.html")]
struct MikrotikEditTemplate {
    mikrotik: Mikrotik,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Mikrotik {
    pub id: i32,
    pub nome: String,
    pub ip: String,
    pub secret: String,
    pub max_clientes: Option<i32>, 
    pub ssh_login: Option<String>,
    //could store this hashed?
    //i think no, its safer
    //it will be used for ssh and doing the fallback logic from radius
    pub ssh_password: Option<String>,
}

#[derive(Template)]
#[template(path = "mikrotik_add.html")]
struct MikrotikFormTemplate;


#[derive(Deserialize , Debug, FromRow)]
pub struct MikrotikDto {
    pub nome: String,
    pub ip: Ipv4Addr,
    pub secret: String,
    pub max_clientes: Option<i32>,
    pub login: Option<String>,
    pub senha: Option<String>,

}

#[derive(Template)]
#[template(path = "mikrotik_list.html")]
struct MikrotikListTemplate {
    mikrotik_options: Vec<Mikrotik>,
}

