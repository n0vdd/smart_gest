use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use reqwest::header::TE;
use tera::Context;
use tracing::{debug, error};
use tracing_subscriber::fmt::writer::Tee;
use std::{net::Ipv4Addr,  str::FromStr, sync::Arc};
use sqlx::PgPool;

use crate::{clientes::{cliente::find_cliente_in_mikrotik, plano::find_all_planos}, provedor::mikrotik_model::Mikrotik, TEMPLATES};

use super::{mikrotik::{delete_mikrotik_db, find_all_mikrotiks, find_mikrotik_by_id, save_mikrotik, update_mikrotik_db}, mikrotik_model::MikrotikDto};

pub async fn add_mikrotik_form() -> impl IntoResponse {
    let template = TEMPLATES.lock().await;
    match template.render("mikrotik/mikrotik_add.html", &tera::Context::new()) {
        Ok(template) => Html(template),
        Err(e) => {
            error!("Failed to render mikrotik_add template: {:?}", e);
            let error = format!("Erro ao renderizar formulario de adicao de mikrotik {e}");
            let mut context = tera::Context::new();
            context.insert("error", &error);
            let template = TEMPLATES.lock().await;
            match template.render("base.html", &context) {
                Ok(template) => Html(template),
                Err(e) => {
                    error!("Failed to render base html: {:?}", e);
                    Html("<p>Failed to render base html</p>".to_string())
                }
            }
        }
    }
}

//Create the script for pasting onto mikrotik with html template?
//Create a modal for displaying this script with a option for copiying
//TODO have a button on the modal to copy its contents
pub async fn failover_mikrotik_script(Path(id):Path<i32>) -> impl IntoResponse {
    let mut script = String::new();
    let ip = local_ip_address::local_ip().map_err(|e| {
        error!("Failed to get local ip: {:?}", e);
        return Html("<p>Failed to get local broadcast ip</p>".to_string())
    }).expect("Failed to get local ip");

    //This should keep it all formmated
    //TODO this should use https for secure connection
    script.push_str(format!(r#"/system scheduler add interval=45m name=ler_pppoe on-event=":execute script=ler_pppoe;
global done "";
/system script add name=ler_pppoe source=" #===============================\r
/tool fetch url="http://{}:8080/mikrotik/{}/faiolver" dst-path=mkt_pppoe.rsc;
:set done "true";
:if ( [/file find name=mkt_pppoe.rsc] != "" ) do={{
:log warning "Importando PPPoE";
/import mkt_pppoe.rsc;
/file remove mkt_pppoe.rsc;
}}
"#,ip.to_string(),id).as_str());
    
    debug!("mikrotik failover script:{}",script);
    script
}


///Generate a script to be downloaded by mikrotik every 45 minutes,creating the .rsc that will delete the previous created faiolver data
///and recreate it, creating the ppp/profiles and ppp/secrets(disabled by default), evereything with the smart_gest comment for easy of scripting
///and so that everyone know that it was created by this system
///param: mikrotik id to get the associated clientes
///param: Db connection for the logic(getting all the planos,linking clientes to mikrotik)
pub async fn failover_radius_script(Path(mikrotik_id):Path<i32>,Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    //Get all the planos
    let planos = find_all_planos(&pool).await.map_err(|e|{
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }).expect("Erro ao buscar planos"); 

    //Get all the clientes for the given mikrotik
    let clientes = find_cliente_in_mikrotik(&pool, mikrotik_id).await.map_err(|e| {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    })
    .expect("Erro ao buscar clientes");

    let mut commands = String::new();
    //remove previous added profiles
    commands.push_str(format!(r#":foreach profile in=[/ppp/profile/find where comment="smart_gest"] do={{/ppp/profile/remove $profile}}
    "#).as_str());
    //remove previous added clientes 
    commands.push_str(format!(r#":foreach cliente in=[/ppp/secret/find where comment="smart_gest"] do={{/ppp/secret/remove $cliente}}
    "#).as_str());

    for plano in &planos {
        debug!("adicionando plano ao script: {:?}",plano);
        //add profiles
        commands.push_str(format!(r#"/ppp/profile/add name="{}" rate-limit={}m/{}m only-one=yes comment="smart_gest"
        "#,
            plano.nome,plano.velocidade_down,plano.velocidade_up).as_str());
    }

    for cliente in &clientes {
        debug!("adicionando cliente ao script: {:?}",cliente);
        let plano_name = planos.iter().find(|plano| plano.id == cliente.plano_id)
            .map(|plano_name| { plano_name.nome.clone() })
            .expect("impossivel achar plano");

        // \$	Output $ character. Otherwise $ is used to link variable.
        // \?	Output ? character. Otherwise ? is used to print "help" in console.
        // \_	- space
        //TODO escape mikrotik problem chars for login and senha
        //We use \\ because the first escapes the second as a rust string
        let login = cliente.login.replace("$","\\$").replace("?","\\?").replace(" ","\\_");
        let senha = cliente.senha.replace("$","\\$").replace("?","\\?").replace(" ","\\_");
        commands.push_str(format!(r#"/ppp/secret/add name="{}" password="{}" profile="{}" service=pppoe comment="smart_gest" disabled=yes
        "#,
        login,senha,plano_name).as_str());
        }

    commands
}

//TODO make the html appear to the user
//Gets the mikrotik id(passed by the button) 
//deletes the related mikrotik
//The redirect is causing error with htmx
pub async fn delete_mikrotik(Path(mikrotik_id):Path<i32>,Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    match delete_mikrotik_db(mikrotik_id, &pool).await {
        Ok(_) => axum::http::StatusCode::OK.into_response(),

        //TODO this error handling method
        Err(e) => {
            error!("Erro ao deletar mikrotik na db {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//Creates a mikrotik instance 
//Receives the form data
//returns a redirect of the user to the mikrotik list
pub async fn register_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<MikrotikDto>,
) -> impl IntoResponse {
    if mikrotik.ip.is_loopback() || mikrotik.ip.is_unspecified() {
        return (axum::http::StatusCode::BAD_REQUEST,"Ip Invalido").into_response();
    }

    match save_mikrotik(&mikrotik, &pool).await {
        Ok(_) => Redirect::to("/mikrotik").into_response(),

        Err(e) => {
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string()).into_response();
        }
    }
}

//Populate and show a list with all created mikrotiks
pub async fn show_mikrotik_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    //Get all created mikrotiks
    //TODO deal with html error
    let mikrotik_list  = find_all_mikrotiks(&pool).await.expect("Erro ao achar mikrotiks na db"); 

    let mut context = tera::Context::new(); 
    context.insert("mikrotik_options", &mikrotik_list);

    let template = TEMPLATES.lock().await;
    match template.render("mikrotik/mikrotik_list.html", &context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render mikrotik_list template: {:?}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Erro ao renderizar lista de mikrotiks").into_response();
        }
    }
}

//Show the edit form for a mikrotik instance
//Uses the id to find it on the db and populate the edit form
pub async fn show_mikrotik_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    //Find the edit instance on the db
    //TODO deal with html error
    let mikrotik = find_mikrotik_by_id(id, &pool).await.expect("Erro ao achar mikrotik na db"); 

    let mut context = Context::new();
    context.insert("mikrotik", &mikrotik);

    let template = TEMPLATES.lock().await;
    match template.render("mikrotik/mikrotik_edit.html",&context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render mikrotik_edit template: {:?}", e);
            let error = format!("Erro ao renderizar formulario de edicao de mikrotik {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
        }
    }
}

//Gets the form with the edited mikrotik data
//saves the changes to the DB
//Return an html with the error
pub async fn update_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<Mikrotik>,
) -> impl IntoResponse {
    //Need to deal with a correct ipv4
    //TODO this could be done on frontend?
    //could just extract it into a function aswell
    let ip = Ipv4Addr::from_str(&mikrotik.ip).map_err(|e| {
            let error = format!("Ipv4 invalido {:?}",e);
            return (axum::http::StatusCode::BAD_REQUEST, error).into_response();
    }).expect("Erro ao validar ipv4"); 

    //Doens not need this ip check, i think there is a case for accepting loopback ips
    if ip.is_loopback() || ip.is_unspecified() {
        let error = "Ip do mikrotik Invalido";
        return (axum::http::StatusCode::BAD_REQUEST,error).into_response();
    }

    match update_mikrotik_db(&mikrotik, &pool).await {
        Ok(_) => Redirect::to("/mikrotik").into_response(),

        Err(e) => {
            let error = format!("Erro ao atualizar mikrotik {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
        }
    }
}

