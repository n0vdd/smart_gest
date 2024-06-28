mod db;
mod handlers;

use axum::routing::put;
use axum::{Router, routing::get, routing::post, extract::Extension};
use handlers::clients::{register_client, show_cliente_form};
use handlers::mikrotik::{register_mikrotik,  show_mikrotik_edit_form, show_mikrotik_form, show_mikrotik_list, update_mikrotik};
use handlers::planos::{register_plano, show_planos_form};
use handlers::utils::{lookup_cep, validate_cpf_cnpj};
use log::debug;
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::db::create_pool;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let pool = Arc::new(create_pool().await.expect("erro ao criar pool"));
    debug!("pool:{:?} criado",pool);

    let app = Router::new()
        .route("/cliente", get(show_cliente_form))
        .route("/cliente", post(register_client))
        .route("/cep", get(lookup_cep))
        .route("/cpf_cnpj", get(validate_cpf_cnpj))
        .route("/mikrotik", get(show_mikrotik_list))
        .route("/mikrotik/add", get(show_mikrotik_form))
        .route("/mikrotik/add", post(register_mikrotik))
        .route("/mikrotik/:id", put(update_mikrotik))
        .route("/mikrotik/:id", get(show_mikrotik_edit_form))
        .route("/plano", get(show_planos_form))
        .route("/plano",post(register_plano))
        .layer(Extension(pool));

    debug!("app:{:?} criado",app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listenet = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listenet,app.into_make_service()).await.expect("erro ao iniciar o servidor");
}
