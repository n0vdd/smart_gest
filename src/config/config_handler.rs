use std::sync::Arc;

use axum::{extract::State, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use sqlx::PgPool;
use tracing::error;

use crate::{financeiro::email_service::setup_email, AppState, TEMPLATES};

use super::{config::{find_email_config, find_nf_config, find_provedor, save_email_config_to_db, save_nf_config_to_db, save_provedor_to_db, update_email_config_in_db, update_nf_config_in_db, update_provedor_in_db}, config_model::{EmailConfig, EmailConfigDto, NfConfig, NfConfigDto, Provedor, ProvedorDto}};

pub async fn show_provedor_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let provedor = find_provedor(&pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar provedor");

    let template = TEMPLATES.lock().await;
    if let Some(provedor) = provedor {
        let mut context = tera::Context::new();
        context.insert("provedor", &provedor);
    
        match template.render("config/provedor_config_edit.html", &context) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Failed to render provedor_config_edit template: {:?}", e);
                let error = format!("Erro ao renderizar provedor_config_edit template: {e}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
            }
        }
    } else {
        match template.render("config/provedor_config_add.html", &tera::Context::new()) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Failed to render provedor_config_add template: {:?}", e);
                let error = format!("Erro ao renderizar provedor_config_add template: {e}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
            }
        }
    }
}

pub async fn save_provedor(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<ProvedorDto>)
    -> impl IntoResponse {
        match save_provedor_to_db(&pool, &provedor).await {
            Ok(_) => Redirect::to("/config/provedor").into_response(),

            Err(e) => {
                error!("Failed to save provedor: {:?}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
}

pub async fn update_provedor(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<Provedor>) 
    -> impl IntoResponse {
    match update_provedor_in_db(&pool, &provedor).await {
        Ok(_) => Redirect::to("/config/provedor").into_response(),

        Err(e) => {
            error!("Failed to update provedor: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//TODO create assas config
//if its using sandbox or production
//the api key for production and sandbox
//?maybe the way to configure the webhook aswell

pub async fn show_nf_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let nf_config = find_nf_config(&pool).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar config de nota fiscal"); 

    let template = TEMPLATES.lock().await;
    if let Some(nf) = nf_config {
        let mut context = tera::Context::new();
        context.insert("id", &nf.id);
        context.insert("contabilidade_email", &nf.contabilidade_email);
    
        match template.render("config/nf_config_edit.html", &context) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Failed to render nf_config_edit template: {:?}", e);
                let erro = format!("Erro ao renderizar nf_config_edit template: {e}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
            }
        }
    } else {
        match template.render("config/nf_config_add.html", &tera::Context::new()) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Failed to render nf_config_add template: {:?}", e);
                let erro = format!("Erro ao renderizar nf_config_add template: {e}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
            }
        }
    }
    
}

pub async fn save_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(nf_config):Form<NfConfigDto>) 
    -> impl IntoResponse {
        match save_nf_config_to_db(&pool, &nf_config).await {
            Ok(_) => Redirect::to("/config/nf").into_response(),

            Err(e) => {
                error!("Failed to save nf_config: {:?}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
}

pub async fn update_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(nf_config):Form<NfConfig>)
    -> impl IntoResponse {
        match update_nf_config_in_db(&pool, &nf_config).await {
            Ok(_) => Redirect::to("/config/nf").into_response(),

            Err(e) => {
                error!("Failed to update nf_config: {:?}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
}

pub async fn show_email_config(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let email_config = find_email_config(&pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar config de email"); 

    let template = TEMPLATES.lock().await;
    if let Some(email_config) = email_config {
        let mut context = tera::Context::new();
        context.insert("email_config", &email_config);
    
        match template.render("config/email_config_edit.html", &context) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Erro ao renderizar template para editar config de email: {:?}", e);
                let erro = format!("Erro ao renderizar template para editar config de email: {e}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
            }
        }
    } else {
        match template.render("config/email_config_add.html", &tera::Context::new()) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Erro ao renderizar template para adicionar config de email: {:?}", e);
                let erro = format!("Erro ao renderizar template para adicionar config de email: {e}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
            }
        }
    }
}

pub async fn save_email_config(Extension(pool):Extension<Arc<PgPool>>,State(mut state):State<AppState>
    ,Form(email_config):Form<EmailConfigDto>) -> impl IntoResponse {
    match save_email_config_to_db(&pool, &email_config).await {
        Ok(_) => {
            //update the emailer on the app state
            let email = setup_email(&pool).await.map_err(|e|
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
            ).expect("Erro ao configurar smtp email");
            state.mailer = Some(email);

            Redirect::to("/config/email").into_response()
        },

        Err(e) => {
            error!("Failed to save email_config: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }   
    }
}

//TODO use htmx to send put and update email_config
//BUG need to check if this is done
pub async fn update_email_config(Extension(pool):Extension<Arc<PgPool>>,State(mut state):State<AppState>
    ,Form(email_config):Form<EmailConfig>) -> impl IntoResponse {
    match update_email_config_in_db(&pool, &email_config).await {
        Ok(_) => {
            //update the emailer on the app state
            let email = setup_email(&pool).await.map_err(|e|
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
            ).expect("Erro ao configurar smtp email");
            state.mailer = Some(email);

            Redirect::to("/config/email").into_response()
        },

        Err(e) => {
            error!("Failed to update email_config: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}