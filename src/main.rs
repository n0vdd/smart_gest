mod db;
mod handlers;
mod services;

use axum::extract::Request;
use axum::http;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{delete, put};
use axum::{Router, routing::get, routing::post, extract::Extension};
use db::create_postgres_pool;
use handlers::clients::{delete_cliente, register_cliente, show_cliente_form, show_cliente_list, update_cliente};
use handlers::contrato::generate_contrato;
use handlers::mikrotik::{delete_mikrotik, register_mikrotik, show_mikrotik_edit_form, show_mikrotik_form, show_mikrotik_list, update_mikrotik};
use handlers::planos::{delete_plano, list_planos, register_plano, show_plano_edit_form, show_planos_form, update_plano};
use handlers::utils::{lookup_cep, validate_cpf_cnpj, validate_phone};
use once_cell::sync::Lazy;
use services::webhooks::{debug, webhook_handler};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

// Define allowed IPs
static ALLOWED_IPS: Lazy<Vec<IpAddr>> = Lazy::new(|| {
    vec![
        IpAddr::V4(Ipv4Addr::new(52, 67, 12, 206)),
        IpAddr::V4(Ipv4Addr::new(18, 230, 8, 159)),
        IpAddr::V4(Ipv4Addr::new(54, 94, 136, 112)),
        IpAddr::V4(Ipv4Addr::new(54, 94, 183, 101)),
        IpAddr::V4(Ipv4Addr::new(54, 207, 175, 46)),
        IpAddr::V4(Ipv4Addr::new(54, 94, 35, 137)),
    ]
});

// Define the valid access token
static VALID_ACCESS_TOKEN: &str = "m+/t\"]9lhtyh{2}s&%Wt";    

//?is this ok?
async fn check_ip<B>(req: Request<B>, next: Next) -> Result<Next, http::StatusCode> {
    let client_ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|value| value.to_str().ok())
        .and_then(|x_forwarded_for| x_forwarded_for.split(',').next())
        .and_then(|ip_str| ip_str.parse().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

    if ALLOWED_IPS.contains(&client_ip) {
        Ok(next)
    } else {
        Err(http::StatusCode::FORBIDDEN)
    }
}

async fn check_access_token<B>(req: Request<B>, next: Next) -> Result<Next, http::StatusCode> {
    let access_token = req
        .headers()
        .get("asaas-access-token")
        .and_then(|value| value.to_str().ok());

    if access_token == Some(VALID_ACCESS_TOKEN) {
        Ok(next)
    } else {
        Err(http::StatusCode::FORBIDDEN)
    }
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    //Setup db
    let pg_pool = Arc::new(create_postgres_pool().await
    .map_err(|e| -> _ {
        error!("erro ao criar pool: {:?}", e);
        panic!("erro ao criar pool")
    }).expect("erro ao criar pool"));
    info!("postgres pool:{:?} criado",pg_pool);

    //prepara templates dos contratos
    handlers::contrato::add_template(&pg_pool).await.map_err(|e| {
        error!("Failed to prepare contrato templates: {:?}", e);
        panic!("Failed to prepare contrato templates")
    }).expect("Failed to prepare contrato templates");


    /*Setup axum-login
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);


    let auth_layer = AuthManagerLayerBuilder::new(,session_layer).build();
    */

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
        .route("/:id",delete(delete_plano));
        //.route("/contrato_template", get(add_template));

    let financial_routes = Router::new()
        .route("/webhook", post(webhook_handler));

    let app = Router::new()
        .nest("/cliente", clientes_routes)
        .nest("/mikrotik", mikrotik_routes)
        .nest("/plano", planos_routes)
        .nest("/financeiro", financial_routes)
        //.nest("/financeiro", financial_routes)
        .layer(Extension(pg_pool))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(&addr).await
    .map_err(|e| -> _ {
        error!("erro ao criar listener: {:?}", e);
        panic!("erro ao criar listener")
    }).expect("erro ao criar listener");

    info!("Listening on {}", addr);
    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await
    .map_err(|e| -> _ {
        error!("erro ao iniciar o servidor: {:?}", e);
        panic!("erro ao iniciar o servidor")
    }).expect("erro ao iniciar o servidor");

}
