use axum::{extract::Form, response::Html, Extension};
use askama::Template;
use std::sync::Arc;
use tokio::sync::Mutex;
use deadpool_postgres::Pool;
use crate::model::ClientData;
//use bcrypt::{hash, DEFAULT_COST};

#[derive(Template)]
#[template(path = "cliente_add.html")]
struct ClienteFormTemplate;

pub async fn show_form() -> Html<String> {
    let template = ClienteFormTemplate;
    Html(template.render().unwrap())
}
pub async fn register_client(
    Extension(pool): Extension<Arc<Mutex<Pool>>>,
    Form(client): Form<ClientData>,
) -> Html<String> {
    let pool = pool.lock().await;

    // Insert the address first
    let endereco_id: i32 = pool.get().await.unwrap().query_one(
        "INSERT INTO enderecos (cep, rua, numero, bairro, complemento, cidade, estado, ibge)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        &[
            &client.endereco.cep,
            &client.endereco.rua,
            &client.endereco.numero,
            &client.endereco.bairro,
            &client.endereco.complemento,
            &client.endereco.cidade,
            &client.endereco.estado,
            &client.endereco.ibge
        ]
    ).await.unwrap().get(0);

    /* 
    let hashed_password = hash(client.password.clone(), DEFAULT_COST).unwrap();
    let client = ClientData {
        password: hashed_password,
        ..client
    };*/
    let _ = pool.get().await.unwrap().execute(
        "INSERT INTO clients (name, email, cpf_cnpj, endereco_id, password, cellphone) VALUES ($1, $2, $3, $4, $5, $6)",
        &[&client.name, &client.email, &client.cpf_cnpj, &endereco_id, &client.password, &client.cellphone],
    ).await;
    Html(format!("<p>Client {} registered successfully!</p>", client.name))
}