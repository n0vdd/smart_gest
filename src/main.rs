mod db;
mod provedor;
mod clientes;
mod financeiro;
mod integracoes;
mod config;

use anyhow::{anyhow, Context};
use axum::extract::Request;
use axum::http;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{delete, put};
use axum::{Router, routing::get, routing::post, extract::Extension};
use chrono::{Datelike, Duration, Utc};
use clientes::cliente::{bloqueia_clientes_atrasados, import_mikauth_clientes};
use clientes::cliente_handler::{bloqueia_cliente_no_radius, delete_cliente, register_cliente, show_cliente_edit_form, show_cliente_form, show_cliente_list, update_cliente};
use clientes::plano_handler::{delete_plano, list_planos, register_plano, show_plano_edit_form, show_planos_form, update_plano};
use config::config_handler::{save_email_config, save_nf_config, save_provedor, show_email_config, show_nf_config, show_provedor_config, update_email_config, update_nf_config, update_provedor};
use config::utils_handler::{lookup_cep, show_endereco, validate_cpf_cnpj, validate_phone};
use cron::Schedule;
use db::create_postgres_pool;
use financeiro::contrato_handler::{add_contrato_template, generate_contrato, show_contrato_template_add_form, show_contrato_template_edit_form, show_contrato_template_list, update_contrato_template};
use financeiro::dici_handler::{generate_dici, generate_dici_month_year, show_dici_list};
use financeiro::email_service::setup_email;
use financeiro::nfs_handler::show_export_lotes_list;
use financeiro::nfs_service::exporta_nfs;
use integracoes::genieacs_voip_service::checa_voip_down;
use integracoes::webhooks_service::webhook_handler;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use once_cell::sync::Lazy;
use provedor::mikrotik_handler::{add_mikrotik_form, delete_mikrotik, failover_mikrotik_script, failover_radius_script, register_mikrotik, show_mikrotik_edit_form, show_mikrotik_list, update_mikrotik};
use radius::{create_radius_cliente_pool, create_radius_plano_bloqueado};
use sqlx::PgPool;
use tera::Tera;
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use lazy_static::lazy_static;
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

lazy_static! {
    pub static ref TEMPLATES: Mutex<Tera> = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                error!("template parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        Mutex::new(tera)
    };
}

#[derive(Clone)]
pub struct AppState {
    pub mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    pub http_client: reqwest::Client,
}

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

//BUG this may be running all the time
//TODO should check every day 1 and 12 of the month
//on day 1 it will call the nfs function that will send the xml of nota fiscal for the month for a email
//will also generate dici for the previous month and send a notification to a telegram channel
//on day 12 will check what clientes have not payd and block them
//when we receive the payment we will unblock the cliente, so there is no need to check again
async fn scheduler(pool: Arc<PgPool> ,state:AppState) -> Result<(), anyhow::Error> {
    let expression = "0 0 0 1,12 * *"; // Run at midnight on the 1st and 12th of every month
    let schedule = Schedule::from_str(expression).expect("Invalid cron expression for scheduler");

    //TODO a expression that will run everyday at 4am
    let daily = "0 5 0 * * *";
    let daily_schedule = Schedule::from_str(&daily).unwrap();

    let mut day_to_come = daily_schedule.upcoming(Utc);
    while let Some(next) = day_to_come.next() {
        let now = Utc::now();
        let duration = next - now;

        if duration > Duration::zero() {
            tokio::time::sleep(duration.to_std().unwrap()).await;
        }

        checa_voip_down(&state.http_client).await.context("Erro ao checar voip")?;
        info!("Voip check executed on {}", next);
    }

    let mut upcoming = schedule.upcoming(Utc);
    while let Some(next) = upcoming.next() {
        let now = Utc::now();
        let duration = next - now;
        if duration > chrono::Duration::zero() {
            tokio::time::sleep(duration.to_std().unwrap()).await;
        }

        // Determine the day of the task
        let day = next.day();

        match day {
            1 => {
                // Call the function to send the nota fiscal XML and generate DICI
                //TODO call code to generate dici and generates telegram notification

                generate_dici_month_year(&pool,now.month(),now.year()).await.map_err(|e| {
                    error!("Failed to generate dici: {:?}", e);
                    anyhow!("Failed to generate dici")
                }).expect("Erro ao gerar dici");

                //Esta salvando para downloads, preciso especificar o local dos downloads
                exporta_nfs(&pool,&state.mailer.clone().unwrap()).await.context("Erro ao exportar NFS")?;
            }
            12 => {
                bloqueia_clientes_atrasados(&pool).await.map_err(|e| {
                    error!("Failed to block overdue clients: {:?}", e);
                    anyhow!("Failed to block overdue clients")
                }).expect("Erro ao bloquear clientes atrasados");
            }
            _ => {
                error!("Unexpected day: {}", day);
            }
        }

        info!("Task executed on {}", next);
    }
    Ok(())
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    /* 
    Command::new("chromedriver").spawn().map_err(|e| {
        error!("Failed to start chromedriver: {:?}", e);
        panic!("Failed to start chromedriver")
    }).expect("Failed to start chromedriver");
*/
    //Setup db
    let pg_pool = Arc::new(create_postgres_pool().await
    .map_err(|e| -> _ {
        error!("erro ao criar pool: {:?}", e);
        panic!("erro ao criar pool")
    }).expect("erro ao criar pool"));

    info!("postgres pool:{:?} criado",pg_pool);

    let mailer = setup_email(&pg_pool).await.ok();

    let state = AppState { mailer, http_client: reqwest::Client::new() };

    /* 
    let mysql_pool = Arc::new(radius::create_mysql_pool().await
    .map_err(|e| -> _ {
        error!("erro ao criar pool: {:?}", e);
        panic!("erro ao criar pool")
    }).expect("erro ao criar pool"));
    info!("mysql pool:{:?} criado",mysql_pool);
    */

    /* 
    import_mikauth_clientes(&pg_pool).await.map_err(|e| {
        error!("Failed to import clientes from mikauth: {:?}", e);
        panic!("Failed to import clientes from mikauth")
    }).expect("Failed to import clientes from mikauth");
    */
    create_radius_cliente_pool().await.map_err(|e| {
        error!("Failed to create radius cliente pool: {:?}", e);
        panic!("Failed to create radius cliente pool")
    }).expect("Failed to create radius cliente pool");

    create_radius_plano_bloqueado().await.map_err(|e| {
        error!("Failed to create radius plano bloqueado: {:?}", e);
        panic!("Failed to create radius plano bloqueado")
    }).expect("Failed to create radius plano bloqueado");

    //BUG this is running all the time 
    //BUG if the mailer is changed this will not be updated?
    tokio::spawn(scheduler(pg_pool.clone(),state.clone()));

    let clientes_routes = Router::new()
        .route("/",get(show_cliente_list))
        .route("/add", get(show_cliente_form))
        .route("/", post(register_cliente))
        .route("/:id", get(show_cliente_edit_form))
        .route("/:id", put(update_cliente))
        .route("/:id", delete(delete_cliente))
        .route("/contrato/:cliente_id", get(generate_contrato))
        .route("/block/:cliente_id", get(bloqueia_cliente_no_radius));

    let mikrotik_routes = Router::new()
        .route("/", get(show_mikrotik_list))
        .route("/add", get(add_mikrotik_form))
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
        .route("/contrato_template", get(show_contrato_template_list))
        .route("/contrato_template", post(add_contrato_template))
        .route("/contrato_template/add", get(show_contrato_template_add_form))
        .route("/contrato_template/:id", get(show_contrato_template_edit_form))
        .route("/contrato_template/:id", put(update_contrato_template))
        .route("/nfs", get(show_export_lotes_list))
        .route("/dici", get(show_dici_list));

    let config_routes = Router::new()
        .route("/nf", get(show_nf_config))
        .route("/nf", post(save_nf_config))
        .route("/nf", put(update_nf_config))
        .route("/email",get(show_email_config))
        .route("/email",post(save_email_config))
        .route("/email",put(update_email_config))
        .route("/provedor", get(show_provedor_config))
        .route("/provedor", post(save_provedor))
        .route("/provedor", put(update_provedor));

    let util_router = Router::new()
        .route("/endereco",get(show_endereco))
        .route("/cep",get(lookup_cep))
        .route("/cpf_cnpj", get(validate_cpf_cnpj))
        .route("/telefone", get(validate_phone));

    //TODO should test the app server
    //maybe use axum_test, maybe use mockall, maybe use the 2 
    let app = Router::new()
        .nest("/cliente", clientes_routes)
        .nest("/mikrotik", mikrotik_routes)
        .nest("/plano", planos_routes)
        .nest("/financeiro", financial_routes)
        .nest("/config", config_routes)
        .nest("/util", util_router)
        //.nest("/financeiro", financial_routes)
        .layer(Extension(pg_pool))
        .with_state(state)
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
    //This is not https
    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await
    .map_err(|e| -> _ {
        error!("erro ao iniciar o servidor: {:?}", e);
        panic!("erro ao iniciar o servidor")
    }).expect("erro ao iniciar o servidor");

}
