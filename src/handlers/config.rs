use std::sync::Arc;

use axum::{extract::State, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use reqwest::StatusCode;
use sqlx::{query, query_as, PgPool};
use tracing::error;

use crate::{models::config::{EmailConfig, EmailConfigDto, NfConfig, NfConfigDto, Provedor, ProvedorDto}, services::email::setup_email, AppState, TEMPLATES};

pub async fn show_provedor_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let provedor = query_as!(Provedor,"SELECT * FROM provedor").fetch_optional(&*pool).await
        .expect("Failed to fetch provedor");

    if provedor.is_none() {
        let template = TEMPLATES.render("provedor_config_add.html", &tera::Context::new())
            .expect("Erro ao renderizar template para adicionar provedor");

        Html(template)
    } else {
        let mut context = tera::Context::new();
        context.insert("provedor",&provedor.unwrap());
        let template = TEMPLATES.render("provedor_config_edit.html", &tera::Context::new())
            .expect("Erro ao renderizar template para editar provedor");

        Html(template)
    }
}

pub async fn save_provedor(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<ProvedorDto>)
    -> impl IntoResponse {
    query!("INSERT INTO provedor (nome,cnpj,cep,rua,numero,bairro,cidade,estado,complemento,telefone,email,observacao) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
        provedor.nome,provedor.cnpj,provedor.endereco.cep,provedor.endereco.rua,provedor.endereco.numero,provedor.endereco.bairro,provedor.endereco.cidade
        ,provedor.endereco.estado,provedor.endereco.complemento,provedor.telefone,provedor.email,provedor.observacao)
        .execute(&*pool).await
        .expect("Failed to insert provedor");

    Redirect::to("/config/provedor")
}

pub async fn update_provedor(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<Provedor>) 
    -> impl IntoResponse {
    query!("UPDATE provedor SET nome = $1, cnpj = $2, cep = $3, rua = $4, numero = $5, bairro = $6, cidade = $7, estado = $8, complemento = $9, telefone = $10, email = $11, observacao = $12 WHERE id = $13",
        provedor.nome,provedor.cnpj,provedor.cep,provedor.rua,provedor.numero,provedor.bairro,provedor.cidade,provedor.estado,provedor.complemento,provedor.telefone,provedor.email,provedor.observacao,provedor.id)
        .execute(&*pool).await
        .expect("Failed to update provedor");

    Redirect::to("/config/provedor")
}


//TODO create assas config
//if its using sandbox or production
//the api key for production and sandbox
//?maybe the way to configure the webhook aswell



pub async fn show_nf_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let nf_config = query_as!(NfConfig,"SELECT * FROM nf_config").fetch_optional(&*pool).await
        .expect("Failed to fetch nf_config");

    if nf_config.is_none() {
        let template = TEMPLATES.render("nf_config_add.html", &tera::Context::new())
            .expect("Erro ao renderizar template para adicionar config de nota fiscal");

        Html(template)
    } else {
        let nf = nf_config.unwrap();
        let mut context = tera::Context::new();
        context.insert("id", &nf.id);
        context.insert("contabilidade_email",& nf.contabilidade_email);

        let template = TEMPLATES.render("nf_config_edit.html", &context)
            .expect("Erro ao renderizar template para editar config de nota fiscal");

        Html(template)
    }
}

pub async fn save_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(nf_config):Form<NfConfigDto>) 
    -> impl IntoResponse {

    query!("INSERT INTO nf_config (contabilidade_email) VALUES ($1)",&nf_config.contabilidade_email)
        .fetch_one(&*pool).await
        .map_err(|e| {
            error!("Failed to insert nf_config: {:?}",e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Failed to insert nf_config");

    Redirect::to("/config/nf")
}

pub async fn update_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(nf_config):Form<NfConfig>)
    -> impl IntoResponse {

    query!("UPDATE nf_config SET contabilidade_email = $1 WHERE id = $2 RETURNING *",&nf_config.contabilidade_email,nf_config.id)
        .fetch_one(&*pool).await
        .map_err(|e| {
            error!("Failed to update nf_config: {:?}",e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Failed to update nf_config");

    Redirect::to("/config/nf")
}

pub async fn show_email_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let email_config = query_as!(EmailConfig,"SELECT * FROM email_config").fetch_optional(&*pool).await
        .expect("Failed to fetch email_config");

    if email_config.is_none() {
        let template = TEMPLATES.render("email_config_add.html", &tera::Context::new())
            .expect("Erro ao renderizar template para adicionar config de email");

        Html(template)
    } else {
        let mut context = tera::Context::new();
        context.insert("email_config",& email_config.unwrap());

        let template = TEMPLATES.render("email_config_edit.html", &context)
            .expect("Erro ao renderizar template para editar config de email");

        Html(template)
    }
}

pub async fn save_email_config(Extension(pool):Extension<Arc<PgPool>>,State(mut state):State<AppState>
    ,Form(email_config):Form<EmailConfigDto>) -> impl IntoResponse {
    query!("INSERT INTO email_config (email,password,host) VALUES ($1,$2,$3)",email_config.email,email_config.password,email_config.host)
        .execute(&*pool).await
        .expect("Failed to insert email_config");
    
    //update the emailer on the app state
    let email = setup_email(&pool).await.expect("Erro ao configurar smtp email");
    state.mailer = Some(email);

    Redirect::to("/config/email")
}

//TODO use htmx to send put and update email_config
//BUG need to check if this is done
pub async fn update_email_config(Extension(pool):Extension<Arc<PgPool>>,State(mut state):State<AppState>
    ,Form(email_config):Form<EmailConfig>) -> impl IntoResponse {
    query!("UPDATE email_config SET email = $1, password = $2, host = $3 WHERE id = $4",email_config.email,email_config.password,email_config.host,email_config.id)
        .execute(&*pool).await
        .expect("Failed to update email_config");

    //update the emailer on the app state
    let email = setup_email(&pool).await.expect("Erro ao configurar smtp email");
    state.mailer = Some(email);

    Redirect::to("/config/email")
}