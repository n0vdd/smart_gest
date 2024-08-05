use std::sync::Arc;

use axum::{extract::Path, response::{Html, IntoResponse}, Extension};
use sqlx::{query_as, PgPool};
use tera::Context;

use crate::{models::nfs::NfLote, TEMPLATES};

pub async fn show_export_lotes_list(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let lotes = query_as!(NfLote,"SELECT * FROM nf_lote").fetch_all(&*pool).await.unwrap();
    let mut context = Context::new();

    context.insert("lotes", &lotes);

    let template = TEMPLATES.render("nota_fiscal_export.html", &context)
    .expect("Failed to render template");

    Html(template)
}

pub async fn envia_lote_contabilidade(Extension(pool):Extension<Arc<PgPool>>,Path(id):Path<i32>)
    -> impl IntoResponse {

    let lotes = query_as!(NfLote,"SELECT * FROM nf_lote WHERE id = $1",id).fetch_one(&*pool).await
        .expect("Failed to fetch lote");

    //TODO send email to contability, will need to have an html email template
    //use letter to send email, will create a service for setup the email and send it,or could use an extension and pass it to this function
    
}