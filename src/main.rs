mod db;
mod model;
mod handlers;

use axum::{Router, routing::get, routing::post, extract::Extension};
use handlers::cep::{lookup_cep, CepService};
use handlers::clients::{register_client, show_cliente_form};
use handlers::mikrotik::{register_mikrotik, show_mikrotik_form};
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
    let cep_service = Arc::new(CepService::new());
    debug!("pool:{:?} criado",pool);

    let app = Router::new()
        .route("/cliente", get(show_cliente_form))
        .route("/cliente", post(register_client))
        .route("/lookup_cep", get(lookup_cep))
        .route("/mikrotik", get(show_mikrotik_form))
        .route("/mikrotik", post(register_mikrotik))
        .layer(Extension(pool))
        .layer(Extension(cep_service));

    debug!("app:{:?} criado",app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listenet = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listenet,app.into_make_service()).await.expect("erro ao iniciar o servidor");
}
