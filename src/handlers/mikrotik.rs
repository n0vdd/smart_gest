use askama::Template;
use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use radius::{create_mikrotik_radius, MikrotikNas};
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::{debug, error};
use std::{net::Ipv4Addr,  str::FromStr, sync::Arc};
use sqlx::{prelude::FromRow, query, query_as, PgPool};

use crate::handlers::{clients::Cliente, planos::Plano};

pub async fn show_mikrotik_form() -> Html<String> {
    let template = MikrotikFormTemplate.render().map_err(|e| {
        error!("Failed to render Mikrotik form template: {:?}", e);
        e
    }).expect("Failed to render Mikrotik form template");

    Html(template)
}

//Create the script for pasting onto mikrotik with html template?
//TODO return inside a html side pane or some shit, with a copy button
//Create a modal for displaying this script with a option for copiying
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
    let planos = query_as!(Plano,"SELECT * FROM planos")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Planos: {:?}", e);
            return Html("<p>Failed to fetch Planos</p>".to_string())
        }).expect("Failed to fetch Planos");

    //Get all the clientes for the given mikrotik
    let clientes = query_as!(Cliente,"SELECT * FROM clientes WHERE mikrotik_id = $1",mikrotik_id)
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Clientes: {:?}", e);
            return Html("<p>Failed to fetch Clientes</p>".to_string())
        }).expect("Failed to fetch Clientes");

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

        //Get the login and senha, escape problematic characters and add the cliente to the mikrotik script
        if let Some(login) = cliente.login.as_ref() {
            // \$	Output $ character. Otherwise $ is used to link variable.
            // \?	Output ? character. Otherwise ? is used to print "help" in console.
            // \_	- space
            //TODO escape mikrotik problem chars for login and senha
            //We use \\ because the first escapes the second as a rust string
            let login = login.replace("$","\\$").replace("?","\\?").replace(" ","\\_");
            if let Some(senha) = cliente.senha.as_ref() {
                let senha = senha.replace("$","\\$").replace("?","\\?").replace(" ","\\_");
                commands.push_str(format!(r#"/ppp/secret/add name="{}" password="{}" profile="{}" service=pppoe comment="smart_gest" disabled=yes
                "#,
                    login,senha,plano_name).as_str());
            }
        }
    }
    commands
}

//TODO make the html appear to the user
//Gets the mikrotik id(passed by the button) 
//deletes the related mikrotik
//The redirect is causing error with htmx
pub async fn delete_mikrotik(Path(mikrotik_id):Path<i32>,Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    query!("DELETE FROM mikrotik where id = $1",mikrotik_id)
        .execute(&*pool)
        .await.map_err(|e| {
            error!("Failed to delete Mikrotik: {:?}", e);
            return Html("<p>Failed to delete Mikrotik</p>".to_string())
        }).expect("Failed to delete Mikrotik");

    Redirect::to("/mikrotik")
}

//Creates a mikrotik instance 
//Receives the form data
//returns a redirect of the user to the mikrotik list
pub async fn register_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<MikrotikDto>,
) -> impl IntoResponse {
    //This shouold not be checked(for virtualized maybe it could be shit)
    if mikrotik.ip.is_loopback() || mikrotik.ip.is_unspecified() {
        return Html("<p>Invalid IP</p>".to_string()).into_response();
    }

    debug!("mikrotik:{:?}",mikrotik);

    query!(
        "INSERT INTO mikrotik (nome, ip, secret, max_clientes)
        VALUES ($1, $2, $3, $4)",
        mikrotik.nome,
        mikrotik.ip.to_string(),
        mikrotik.secret,
        mikrotik.max_clientes
    )
    .execute(&*pool)
    .await
    .map_err(|e| -> _ {
        debug!("Failed to insert Mikrotik: {:?}", e);
        return Html("<p>Failed to insert Mikrotik</p>".to_string())
    }).expect("Failed to insert Mikrotik");

    let mikrotik_nas = MikrotikNas {
        nasname: mikrotik.ip.to_string(),
        shortname: mikrotik.nome,
        secret: mikrotik.secret,
    };

    create_mikrotik_radius(mikrotik_nas).await.map_err(|e| {
        error!("Falha ao adicionar mikrotik para tabela nas da db radius {e}");
        return Html("<p>Falha ao adicionar mikrotik ao radius</p>")
    }).expect("Falha ao adicionar mikrotik a tabela nas do radius");
    Redirect::to("/mikrotik").into_response()
}


//Populate and show a list with all created mikrotiks
pub async fn show_mikrotik_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    //Get all created mikrotiks
    let mikrotik_list  = query_as!(Mikrotik,"SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Mikrotik: {:?}", e);
            return Html("<p>Failed to fetch Mikrotik</p>".to_string())
        }).expect("Failed to fetch Mikrotik");

    //Populate the template with the mikrotik instances
    let template = MikrotikListTemplate {
        mikrotik_options: mikrotik_list,
    }.render().map_err(|e| -> _ {
        error!("Failed to render Mikrotik list template: {:?}", e);
        return Html("<p>Failed to render Mikrotik list template</p>".to_string())
    }).expect("Failed to render Mikrotik list template");

    Html(template)
}

//Show the edit form for a mikrotik instance
//Uses the id to find it on the db and populate the edit form
pub async fn show_mikrotik_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    //Find the edit instance on the db
    let mikrotik = query_as!(Mikrotik,"SELECT * FROM mikrotik WHERE id = $1",id)
        .fetch_one(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Mikrotik for editing: {:?}", e);
            //maybe change this to a Did not find the mikrotik instace for editing or some shit
            return Html("<p>Failed to fetch Mikrotik</p>".to_string())
        }).expect("Failed to fetch Mikrotik");

    //Populate the edit form template with the mikrotik data
    let template = MikrotikEditTemplate {
        mikrotik,
    }.render().map_err(|e| -> _ {
        error!("Failed to render Mikrotik edit template: {:?}", e);
        return Html("<p>Failed to render Mikrotik edit template</p>".to_string())
    }).expect("Failed to render Mikrotik edit template");

    Html(template)
}

//Gets the form with the edited mikrotik data
//saves the changes to the DB
//Return an html with the error
//TODO make better error messages on the frontend(modals and shit)
//Or returns a redirect of the user to the list of mikrotiks, when it all goes well
pub async fn update_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<Mikrotik>,
) -> impl IntoResponse {
    //Need to deal with a correct ipv4
    //TODO daqui uns anos deveria lidar com a possibilidade de ipv6?
    let ip = Ipv4Addr::from_str(&mikrotik.ip)
    .map_err(|e| -> _ {
        error!("Failed to parse IP: {:?}", e);
        return Html("<p>Failed to parse IP</p>".to_string())
    }).expect("Failed to parse IP");

    //Doens not need this ip check, i think there is a case for accepting loopback ips
    if ip.is_loopback() || ip.is_unspecified() {
        return Html("<p>Invalid IP</p>".to_string()).into_response();
    }

    query!(
        "UPDATE mikrotik SET nome = $1, ip = $2, secret = $3, max_clientes = $4 WHERE id = $5",
        mikrotik.nome,
        mikrotik.ip,
        mikrotik.secret,
        mikrotik.max_clientes,
        mikrotik.id
    ).execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to update Mikrotik: {:?}", e);
        return Html("<p>Failed to update Mikrotik</p>".to_string())
    }).expect("Failed to update Mikrotik");

    Redirect::to("/mikrotik").into_response()
}

#[derive(Template)]
#[template(path = "mikrotik_edit.html")]
struct MikrotikEditTemplate {
    mikrotik: Mikrotik,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Mikrotik {
    pub id: i32,
    pub nome: String,
    pub ip: String,
    pub secret: String,
    pub max_clientes: Option<i32>, 
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>,
}

#[derive(Template)]
#[template(path = "mikrotik_add.html")]
struct MikrotikFormTemplate;


#[derive(Deserialize , Debug, FromRow)]
pub struct MikrotikDto {
    pub nome: String,
    pub ip: Ipv4Addr,
    pub secret: String,
    pub max_clientes: Option<i32>
}

#[derive(Template)]
#[template(path = "mikrotik_list.html")]
struct MikrotikListTemplate {
    mikrotik_options: Vec<Mikrotik>,
}

