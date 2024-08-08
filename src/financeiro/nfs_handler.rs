use std::sync::Arc;

use axum::{extract::Path, response::{Html, IntoResponse}, Extension};
use sqlx::PgPool;
use tera::Context;
use tracing::error;


use crate::TEMPLATES;

use super::nfs::{find_all_nfs_lotes, find_nf_lote_by_id};


pub async fn show_export_lotes_list(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let lotes = find_all_nfs_lotes(&pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Failed to fetch lotes");
    let mut context = Context::new();

    context.insert("lotes", &lotes);

    match TEMPLATES.render("financeiro/nota_fiscal_export.html", &context) {
        Ok(template) => Html(template),

        Err(e) => {
            error!("Failed to render nota_fiscal_export template: {:?}", e);
            Html("<p>Failed to render nota_fiscal_export template</p>".to_string())
        }
    }
}

pub async fn envia_lote_contabilidade(Extension(pool):Extension<Arc<PgPool>>,Path(id):Path<i32>)
    -> impl IntoResponse {

    let lotes = find_nf_lote_by_id(&pool, id).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Failed to fetch lotes"); 

    //TODO send email to contability, will need to have an html email template
    //use letter to send email, will create a service for setup the email and send it,or could use an extension and pass it to this function
    
}