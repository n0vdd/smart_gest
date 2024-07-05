mod db;
mod handlers;
mod services;

use axum::routing::{delete, put};
use axum::{Router, routing::get, routing::post, extract::Extension};
use db::create_postgres_pool;
use handlers::clients::{delete_cliente, register_cliente, show_cliente_form, show_cliente_list, update_cliente};
use handlers::contrato::{add_template, generate_contrato};
use handlers::mikrotik::{delete_mikrotik, register_mikrotik, show_mikrotik_edit_form, show_mikrotik_form, show_mikrotik_list, update_mikrotik};
use handlers::planos::{delete_plano, list_planos, register_plano, show_plano_edit_form, show_planos_form, update_plano};
use handlers::utils::{lookup_cep, validate_cpf_cnpj, validate_phone};
use log::{error, info};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::sync::Arc;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let pg_pool = Arc::new(create_postgres_pool().await
    .map_err(|e| -> _ {
        error!("erro ao criar pool: {:?}", e);
        panic!("erro ao criar pool")
    }).expect("erro ao criar pool"));


    info!("postgres pool:{:?} criado",pg_pool);

    let clientes_routes = Router::new()
        .route("/",get(show_cliente_list))
        .route("/add", get(show_cliente_form))
        .route("/add", post(register_cliente))
        .route("/:id", put(update_cliente))
        .route("/:id", delete(delete_cliente))
        .route("/contrato/:cliente_id", get(generate_contrato))
        .route("/cep", get(lookup_cep))
        .route("/telefone", get(validate_phone))
        .route("/cpf_cnpj", get(validate_cpf_cnpj));

    let mikrotik_routes = Router::new()
        .route("/", get(show_mikrotik_list))
        .route("/add", get(show_mikrotik_form))
        .route("/add", post(register_mikrotik))
        .route("/:id", put(update_mikrotik))
        .route("/:id", delete(delete_mikrotik))
        .route("/:id", get(show_mikrotik_edit_form));

    let planos_routes = Router::new()
        .route("/", get(list_planos))
        .route("/add", get(show_planos_form))
        .route("/add",post(register_plano))
        .route("/:id", get(show_plano_edit_form))
        .route("/:id",put(update_plano))
        .route("/:id",delete(delete_plano))
        .route("/contrato_template", get(add_template));


    let app = Router::new()
        .nest("/cliente", clientes_routes)
        .nest("/mikrotik", mikrotik_routes)
        .nest("/plano", planos_routes)
        .layer(Extension(pg_pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(&addr).await
    .map_err(|e| -> _ {
        error!("erro ao criar listener: {:?}", e);
        panic!("erro ao criar listener")
    }).expect("erro ao criar listener");

    info!("Listening on {}", addr);
    axum::serve(listener,app.into_make_service()).await
    .map_err(|e| -> _ {
        error!("erro ao iniciar o servidor: {:?}", e);
        panic!("erro ao iniciar o servidor")
    }).expect("erro ao iniciar o servidor");
}
