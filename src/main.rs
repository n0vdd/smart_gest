mod db;
mod handlers;
mod services;

use anyhow::anyhow;
use axum::extract::Request;
use axum::http;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{delete, put};
use axum::{Router, routing::get, routing::post, extract::Extension};
use cron::Schedule;
use db::create_postgres_pool;
use handlers::clients::{delete_cliente, get_all_clientes, register_cliente, show_cliente_form, show_cliente_list, update_cliente};
use handlers::contrato::generate_contrato;
use handlers::dici::{generate_dici, show_dici_list};
use handlers::mikrotik::{delete_mikrotik, failover_mikrotik_script, failover_radius_script, register_mikrotik, show_mikrotik_edit_form, show_mikrotik_form, show_mikrotik_list, update_mikrotik};
use handlers::planos::{delete_plano, list_planos, register_plano, show_plano_edit_form, show_planos_form, update_plano};
use handlers::utils::{lookup_cep, show_endereco, validate_cpf_cnpj, validate_phone};
use once_cell::sync::Lazy;
use radius::radius::{bloqueia_cliente, create_radius_cliente_pool, create_radius_plano_bloqueado};
use services::webhooks::webhook_handler;
use sqlx::{query, PgPool};
use time::macros::format_description;
use time::PrimitiveDateTime;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
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

//Check the valid asaas-ips
async fn check_ip(req: Request,next:Next) -> Result<impl IntoResponse,http::StatusCode> {
    let client_ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|value| value.to_str().ok())
        .and_then(|x_forwarded_for| x_forwarded_for.split(',').next())
        .and_then(|ip_str| ip_str.parse().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

    debug!("Client IP: {:?}", client_ip);
    debug!("Allowed IPs: {:?}", ALLOWED_IPS);

    if ALLOWED_IPS.contains(&client_ip) {
        let res = next.run(req).await;
        Ok(res)
    } else {
        error!("Invalid IP: {:?}", client_ip);
        Err(http::StatusCode::FORBIDDEN)
    }
}

async fn check_access_token(req: Request,next: Next) -> Result<impl IntoResponse, http::StatusCode> {
    let access_token = req
        .headers()
        .get("asaas-access-token")
        .and_then(|value| value.to_str().ok());

    if let Some(access_token) = access_token {
        if access_token == VALID_ACCESS_TOKEN {
            let req = next.run(req).await;
            Ok(req)
        } else {
            error!("Invalid access token: {:?}", access_token);
            Err(http::StatusCode::FORBIDDEN)
        }
    } else {
        error!("Missing access token");
        Err(http::StatusCode::FORBIDDEN)
    }
}

async fn scheduler(pool:Arc<PgPool>) {
    // Define the cron expression for the 12th day of every month at midnight
    let expression = "0 0 0 12 * *"; // Seconds Minutes Hours Day-of-Month Month Day-of-Week

    let schedule = Schedule::from_str(expression).expect("Erro ao criar scheduler para bloqueio de cliente");

    let mut upcoming = schedule.upcoming(chrono::Utc);
    while let Some(next) = upcoming.next() {
        let duration = next - chrono::Utc::now();
        info!("Next run at {:?}", next);

        if duration > chrono::Duration::zero() {
            //TODO check all the clients that have not paid and block them
            //could check by the db with payment confirmed/received
            let clientes = get_all_clientes(&*pool).await.map_err(|e| {
                error!("Failed to fetch clientes: {:?}", e);
                anyhow!("Failed to fetch all clientes")
            }).expect("Erro ao buscar clientes");

            for cliente in clientes {
                //TODO check the cliente_id inside the payment confirmed for the current month
                //Format chrono to primitiveDateTive
                let format = format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
                let date = PrimitiveDateTime::parse(chrono::Utc::now().to_string().as_str(), format).expect("Erro ao formatar data");

                //BUG maybe the way i am cheking the date is not the best
                let payment = query!("SELECT * FROM payment_confirmed where cliente_id = $1 and created_at >= $2 and created_at <= $3",cliente.id,date,date)
                .fetch_optional(&*pool).await.map_err(|e| {
                    error!("Failed to fetch payment_confirmed: {:?}", e);
                    anyhow!("Failed to fetch all payment_confirmed")
                }).expect("Erro ao buscar payment_confirmed");

                if payment.is_none() {
                    bloqueia_cliente(cliente.login).await.map_err(|e| {
                        error!("Failed to block cliente: {:?}", e);
                        anyhow!("Failed to block cliente")
                    }).expect("Erro ao bloquear cliente");
                }

            }

        }
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

    /* 
    let mysql_pool = Arc::new(radius::create_mysql_pool().await
    .map_err(|e| -> _ {
        error!("erro ao criar pool: {:?}", e);
        panic!("erro ao criar pool")
    }).expect("erro ao criar pool"));
    info!("mysql pool:{:?} criado",mysql_pool);
    */


    create_radius_cliente_pool().await.map_err(|e| {
        error!("Failed to create radius cliente pool: {:?}", e);
        panic!("Failed to create radius cliente pool")
    }).expect("Failed to create radius cliente pool");

    create_radius_plano_bloqueado().await.map_err(|e| {
        error!("Failed to create radius plano bloqueado: {:?}", e);
        panic!("Failed to create radius plano bloqueado")
    }).expect("Failed to create radius plano bloqueado");

    //prepara templates dos contratos
    handlers::contrato::add_template(&pg_pool).await.map_err(|e| {
        error!("Failed to prepare contrato templates: {:?}", e);
        panic!("Failed to prepare contrato templates")
    }).expect("Failed to prepare contrato templates");

    tokio::spawn(scheduler(pg_pool.clone()));

    let clientes_routes = Router::new()
        .route("/",get(show_cliente_list))
        .route("/add", get(show_cliente_form))
        .route("/add", post(register_cliente))
        .route("/:id", put(update_cliente))
        .route("/:id", delete(delete_cliente))
        .route("/contrato/:cliente_id", get(generate_contrato))
        .route("/cep", get(lookup_cep))
        .route("/telefone", get(validate_phone))
        .route("/cpf_cnpj", get(validate_cpf_cnpj))
        .route("/endereco",get(show_endereco));

    let mikrotik_routes = Router::new()
        .route("/", get(show_mikrotik_list))
        .route("/add", get(show_mikrotik_form))
        .route("/add", post(register_mikrotik))
        .route("/:id", put(update_mikrotik))
        .route("/:id", delete(delete_mikrotik))
        //TODO test this
        .route("/radius/:id",get(failover_mikrotik_script))
        //TODO test this
        .route("/:id/faiolver", get(failover_radius_script))
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
        .route("/webhook", post(webhook_handler))
        
        .layer(ServiceBuilder::new()
//              .layer(axum::middleware::from_fn(check_ip))
                .layer(axum::middleware::from_fn(check_access_token))
        )
        .route("/generate_dici",post(generate_dici))
        .route("/dici", get(show_dici_list));

    let app = Router::new()
        .nest("/cliente", clientes_routes)
        .nest("/mikrotik", mikrotik_routes)
        .nest("/plano", planos_routes)
        .nest("/financeiro", financial_routes)
        //.nest("/financeiro", financial_routes)
        .layer(Extension(pg_pool))
//        .layer(Extension(mysql_pool))
        .layer(TraceLayer::new_for_http());
    //TODO add cors 

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

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
