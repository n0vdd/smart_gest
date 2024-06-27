use axum::{response::Html, Extension};
use askama::Template;
use axum_extra::extract::Form;
use cnpj::Cnpj;
use cpf::Cpf;
use std::sync::Arc;
use sqlx::PgPool;
use crate::model::ClientData;

#[derive(Template)]
#[template(path = "cliente_add.html")]
struct ClienteFormTemplate;

pub async fn show_cliente_form() -> Html<String> {
    let template = ClienteFormTemplate;
    Html(template.render().unwrap())
}

pub async fn register_client(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(client): Form<ClientData>,
) -> Html<String> {
    let endereco_id: (i32,) = sqlx::query_as(
        "INSERT INTO enderecos (cep, street, number, neighborhood, complement, city, state, ibge_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
    )
    .bind(&client.endereco.cep.cep)
    .bind(&client.endereco.rua)
    .bind(&client.endereco.numero)
    .bind(&client.endereco.bairro)
    .bind(&client.endereco.complemento)
    .bind(&client.endereco.cidade)
    .bind(&client.endereco.estado)
    .bind(&client.endereco.ibge)
    .fetch_one(&*pool)
    .await
    .expect("Failed to insert endereco");

    // let hashed_password = hash(client.password.clone(), DEFAULT_COST).unwrap();
    // let client = ClientData {
    //     password: hashed_password,
    //     ..client
    // };

    let mut valid_cpf = String::new();
    let mut valid_cnpj = String::new();
    //TODO: validate cpf_cnpj
    match client.pf_or_pj {
        true => {
            valid_cpf = client.cpf_cnpj.parse::<Cpf>().expect("error parsing cpf").to_string();
        },
        false => {
            valid_cnpj = client.cpf_cnpj.parse::<Cnpj>().expect("error parsing cnpj").to_string();
        }
    }

    //TODO will need to redo the saving logic here aswell?
    //it would be better to save cpf/cnpj on the frontend(can use htmx)
    //and just expose a validator endpoint
    //i should also save the cpf/cnpj formatted and unfomatted on the db
    //for nfs generation i need unformated but for contract generation i will need it formated

    sqlx::query(
        "INSERT INTO clients (pf_or_pj, name, email, cpf_cnpj, rg, cellphone, phone, login, password, address_id, mikrotik_id, plan_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
    )
    .bind(client.pf_or_pj)
    .bind(&client.name)
    .bind(&client.email)
    .bind(&client.cpf_cnpj)
    .bind(&client.cellphone)
    .bind(&client.login)
    .bind(&client.password)
    .bind(endereco_id.0)
    .bind(client.mikrotik_id)
    .bind(client.plan_id)
    .execute(&*pool)
    .await
    .expect("Failed to insert client");

    Html(format!("<p>Client {} registered successfully!</p>", client.name))
}
