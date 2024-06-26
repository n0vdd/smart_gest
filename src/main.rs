mod db;
mod model;
mod handlers;

use axum::{Router, routing::get, routing::post, extract::Extension};
use handlers::cep::{lookup_cep, CepService};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::handlers::clients::{show_form, register_client};
use crate::db::create_pool;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let pool = Arc::new(create_pool().await.expect("erro ao criar pool"));


    let app = Router::new()
        .route("/", get(show_form))
        .route("/register", post(register_client))
        .route("/lookup_cep", get(lookup_cep))
        .layer(Extension(pool))
        .layer(Extension(CepService::new()));


    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listenet = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listenet,app.into_make_service()).await.expect("erro ao iniciar o servidor");
}
