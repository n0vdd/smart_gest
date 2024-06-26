mod db;
mod model;
mod handlers;

use axum::serve;
use axum::{Router, routing::get, routing::post, extract::Extension};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::handlers::clients::{show_form, register_client};
use crate::db::create_pool;


#[tokio::main]
async fn main() {
    let pool = Arc::new(Mutex::new(create_pool().await.unwrap()));

    let app = Router::new()
        .route("/", get(show_form))
        .route("/register", post(register_client))
        .layer(Extension(pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listenet = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listenet,app.into_make_service()).await.expect("erro ao iniciar o servidor");
}
