use askama::Template;
use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::{debug, error};
use std::{io::{Read, Write}, net::{Ipv4Addr, TcpStream},  str::FromStr, sync::Arc};
use sqlx::{prelude::FromRow, query, query_as, PgPool};

use crate::handlers::{clients::{self, Cliente}, planos::Plano};

pub async fn show_mikrotik_form() -> Html<String> {
    let template = MikrotikFormTemplate;
    Html(template.render().expect("Failed to render Mikrotik form template"))
}

//Create the script with html template?
pub async fn failover_radius(Path(mikrotik_id):Path<i32>,Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
    let planos = query_as!(Plano,"SELECT * FROM planos")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Planos: {:?}", e);
            return Html("<p>Failed to fetch Planos</p>".to_string())
        }).expect("Failed to fetch Planos");

    let clientes = query_as!(Cliente,"SELECT * FROM clientes WHERE mikrotik_id = $1",mikrotik_id)
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch Clientes: {:?}", e);
            return Html("<p>Failed to fetch Clientes</p>".to_string())
        }).expect("Failed to fetch Clientes");

    let mut commands = String::new();
    //remove previous profiles
    commands.push_str(format!(r#":foreach profile in=[/ppp/profile/find where comment="smart_gest"] do={{/ppp/profile/remove $profile}}
    "#).as_str());
    //remove previous clientes 
    commands.push_str(format!(r#":foreach cliente in=[/ppp/secret/find where comment="smart_gest"] do={{/ppp/secret/remove $cliente}}
    "#).as_str());

    for plano in &planos {
        debug!("adicionando plano ao script: {:?}",plano);
        //TODO criar planos sem ser com mbs, terei que alterar aqui
        //add profiles
        commands.push_str(format!(r#"/ppp/profile/add name="{}" rate-limit={}m/{}m only-one=yes comment="smart_gest"
        "#,
            plano.nome,plano.velocidade_down,plano.velocidade_up).as_str());
    }

    for cliente in &clientes {
        debug!("adicionando cliente ao script: {:?}",cliente);
        let plano_name = planos.iter().find(|plano| plano.id == cliente.plano_id.expect("impossivel achar plano"))
            .map(|plano_name| { plano_name.nome.clone() })
            .expect("impossivel achar plano");

        //add clients

        //TODO escape mikrotik problem chars(can deal with the optional login and senha better and do this in the same part of the code)
        commands.push_str(format!(r#"/ppp/secret/add name="{}" password="{}" profile="{}" service=pppoe comment="smart_gest" disabled=yes
        "#,
            cliente.login.as_ref().expect("impossivel encontrar login"),cliente.senha.as_ref().expect("impossivel achar senha"),plano_name).as_str());
    }

    commands
}

//TODO make the html appear to the user
pub async fn delete_mikrotik(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    query!("DELETE FROM mikrotik")
        .execute(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to delete Mikrotik: {:?}", e);
            return Html("<p>Failed to delete Mikrotik</p>".to_string())
        }).expect("Failed to delete Mikrotik");

    Redirect::to("/mikrotik")
}

pub async fn register_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<MikrotikDto>,
) -> impl IntoResponse {
    if mikrotik.ip.is_loopback() || mikrotik.ip.is_unspecified() {
        return Html("<p>Invalid IP</p>".to_string()).into_response();
    }

    debug!("mikrotik:{:?}",mikrotik);

    //how do i make this check at compile time?
    query!(
        "INSERT INTO mikrotik (nome, ip, secret, max_clientes, ssh_login, ssh_password)
        VALUES ($1, $2, $3, $4, $5, $6)",
        mikrotik.nome,
        mikrotik.ip.to_string(),
        mikrotik.secret,
        mikrotik.max_clientes,
        mikrotik.login,
        mikrotik.senha)
    .execute(&*pool)
    .await
    .map_err(|e| -> _ {
        debug!("Failed to insert Mikrotik: {:?}", e);
        Html("<p>Failed to insert Mikrotik</p>".to_string())
    }).expect("Failed to insert Mikrotik");

    Redirect::to("/mikrotik").into_response()
}

pub async fn show_mikrotik_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let mikrotik_list  = query_as!(Mikrotik,"SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            debug!("Failed to fetch Mikrotik: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        }).expect("Failed to fetch Mikrotik");

    let template = MikrotikListTemplate {
        mikrotik_options: mikrotik_list,
    };

    let template = template.render().map_err(|e| -> _ {
        error!("Failed to render Mikrotik list template: {:?}", e);
        Html("<p>Failed to render Mikrotik list template</p>".to_string())
    }).expect("Failed to render Mikrotik list template");

    Html(template)
}

pub async fn show_mikrotik_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {
    let mikrotik = query_as!(Mikrotik,"SELECT * FROM mikrotik WHERE id = $1",id)
        .fetch_one(&*pool)
        .await.map_err(|e| -> _ {
            debug!("Failed to fetch Mikrotik for editing: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        }).expect("Failed to fetch Mikrotik");

    let template = MikrotikEditTemplate {
        mikrotik,
    };

    let template = template.render().map_err(|e| -> _ {
        error!("Failed to render Mikrotik edit template: {:?}", e);
        Html("<p>Failed to render Mikrotik edit template</p>".to_string())
    }).expect("Failed to render Mikrotik edit template");

    Html(template)
}

pub async fn update_mikrotik(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mikrotik): Form<Mikrotik>,
) -> impl IntoResponse {
    let ip = Ipv4Addr::from_str(&mikrotik.ip)
    .map_err(|e| -> _ {
        error!("Failed to parse IP: {:?}", e);
        Html("<p>Failed to parse IP</p>".to_string())
    }).expect("Failed to parse IP");

    if ip.is_loopback() || ip.is_unspecified() {
        return Html("<p>Invalid IP</p>".to_string()).into_response();
    }

    query!(
        "UPDATE mikrotik SET nome = $1, ip = $2, secret = $3, max_clientes = $4, ssh_login = $5, ssh_password = $6 WHERE id = $7",
        mikrotik.nome,
        mikrotik.ip,
        mikrotik.secret,
        mikrotik.max_clientes,
        mikrotik.ssh_login,
        mikrotik.ssh_password,
        mikrotik.id
    ).execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to update Mikrotik: {:?}", e);
        Html("<p>Failed to update Mikrotik</p>".to_string())
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
    pub ssh_login: Option<String>,
    //could store this hashed?
    //i think no, its safer
    //it will be used for ssh and doing the fallback logic from radius
    pub ssh_password: Option<String>,
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
    pub max_clientes: Option<i32>,
    pub login: Option<String>,
    pub senha: Option<String>,
}

#[derive(Template)]
#[template(path = "mikrotik_list.html")]
struct MikrotikListTemplate {
    mikrotik_options: Vec<Mikrotik>,
}

