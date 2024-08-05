use std::sync::Arc;

use axum::{extract::State, response::{Html, IntoResponse}, Extension};
use axum_extra::extract::Form;
use sqlx::{query, query_as, PgPool};

use crate::{models::config::{EmailConfig, EmailConfigDto, NfConfig, NfConfigDto, Provedor, ProvedorDto}, services::email::{self, setup_email}, AppState, TEMPLATES};

//TODO this will show the form to create/edit the provedor of the system
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

pub async fn save_provedor(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<ProvedorDto>) {
    query!("INSERT INTO provedor (nome,cnpj,cep,rua,numero,bairro,cidade,estado,complemento,telefone,email,observacao) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
        provedor.nome,provedor.cnpj,provedor.cep,provedor.rua,provedor.numero,provedor.bairro,provedor.cidade,provedor.estado,provedor.complemento,provedor.telefone,provedor.email,provedor.observacao)
        .execute(&*pool).await
        .expect("Failed to insert provedor");
}

pub async fn update_provedor(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<Provedor>) {
    query!("UPDATE provedor SET nome = $1, cnpj = $2, cep = $3, rua = $4, numero = $5, bairro = $6, cidade = $7, estado = $8, complemento = $9, telefone = $10, email = $11, observacao = $12 WHERE id = $13",
        provedor.nome,provedor.cnpj,provedor.cep,provedor.rua,provedor.numero,provedor.bairro,provedor.cidade,provedor.estado,provedor.complemento,provedor.telefone,provedor.email,provedor.observacao,provedor.id)
        .execute(&*pool).await
        .expect("Failed to update provedor");
}

pub async fn show_nf_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    //TODO show the form to configure the NF
    let nf_config = query_as!(NfConfig,"SELECT * FROM nf_config").fetch_optional(&*pool).await
        .expect("Failed to fetch nf_config");

    if nf_config.is_none() {
        let template = TEMPLATES.render("nf_config_add.html", &tera::Context::new())
            .expect("Erro ao renderizar template para adicionar config de nota fiscal");

        Html(template)
    } else {
        let mut context = tera::Context::new();
        context.insert("email",& nf_config.unwrap().contabilidade_email);

        let template = TEMPLATES.render("nf_config_edit.html", &context)
            .expect("Erro ao renderizar template para editar config de nota fiscal");

        Html(template)
    }
}

pub async fn save_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(nf_config):Form<NfConfigDto>) {
    query!("INSERT INTO nf_config (contabilidade_email) VALUES ($1)",nf_config.contabilidade_email)
        .execute(&*pool).await
        .expect("Failed to insert nf_config");
}

pub async fn update_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(nf_config):Form<NfConfig>) {
    query!("UPDATE nf_config SET contabilidade_email = $1 WHERE id = $2",nf_config.contabilidade_email,nf_config.id)
        .execute(&*pool).await
        .expect("Failed to update nf_config");
}

pub async fn show_email_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    //TODO show the form to configure the email
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

pub async fn save_email_config(Extension(pool):Extension<Arc<PgPool>>,State(mut state):State<AppState>,Form(email_config):Form<EmailConfigDto>) {
    query!("INSERT INTO email_config (email,password,host) VALUES ($1,$2,$3)",email_config.email,email_config.password,email_config.host)
        .execute(&*pool).await
        .expect("Failed to insert email_config");
    
    //update the emailer on the app state
    let email = setup_email(&pool).await.expect("Erro ao configurar smtp email");
    state.mailer = Some(email);
}

pub async fn update_email_config(Extension(pool):Extension<Arc<PgPool>>,State(mut state):State<AppState>,Form(email_config):Form<EmailConfig>) {
    query!("UPDATE email_config SET email = $1, password = $2, host = $3 WHERE id = $4",email_config.email,email_config.password,email_config.host,email_config.id)
        .execute(&*pool).await
        .expect("Failed to update email_config");

    //update the emailer on the app state
    let email = setup_email(&pool).await.expect("Erro ao configurar smtp email");
    state.mailer = Some(email);
}