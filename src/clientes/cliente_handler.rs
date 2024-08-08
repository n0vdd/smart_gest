use axum::{extract::{Path, State}, response::{Html, IntoResponse, Redirect}, Extension};
use radius::{bloqueia_cliente_radius, add_cliente_radius, ClienteNas};
use tracing::error;
use axum_extra::extract::Form;
use cnpj::Cnpj;
use cpf::Cpf;
use std::sync::Arc ;
use sqlx::PgPool;

use crate::{integracoes::webhooks_service::add_cliente_to_asaas, provedor::mikrotik::find_all_mikrotiks, AppState, TEMPLATES};

use super::{cliente::{delete_cliente_by_id, find_all_clientes, get_cliente_login_by_id, save_cliente, update_cliente_by_id}, cliente_model::{Cliente, ClienteDto, TipoPessoa}, plano::{find_all_planos, find_plano_by_id}};



pub async fn bloqueia_cliente_no_radius(Extension(pool):Extension<Arc<PgPool>>,Path(id): Path<i32>) -> impl IntoResponse {
    let login = get_cliente_login_by_id(&pool, id).await.expect("Erro ao buscar login do cliente");

    match bloqueia_cliente_radius(&login).await {
        Ok(_) => Redirect::to("/cliente").into_response(),

        Err(e) => {
            error!("Failed to block cliente: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//Get all the clientes from the db
//Render the template with the clientes
//return the client list
pub async fn show_cliente_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {

    let clientes = find_all_clientes(&pool).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))
        .expect("Erro ao buscar clientes");

    let mut context = tera::Context::new();
    context.insert("clients", &clientes);

    match TEMPLATES.render("cliente/cliente_list.html", &context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render client list template: {:?}", e);
            let erro = format!("Erro ao renderizar lista de clientes {:?}",e.kind);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
        }
    }
}

/* TODO deal with edit form later
//lets look what the rest of the things we have to do
//need to configure radius and the importante shit
pub async fn show_cliente_edit_form(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Html<String> {

    let client = query_as!(Cliente, "SELECT * FROM clientes WHERE id = $1", id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch client: {:?}", e);
            Html("<p>Failed to fetch client</p>".to_string())
        })
        .expect("Failed to fetch client");

    let mikrotik_list = query_as!(Mikrotik, "SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch Mikrotik: {:?}", e);
            Html("<p>Failed to fetch Mikrotik</p>".to_string())
        })
        .expect("Failed to fetch Mikrotik");

    let plan_list = query_as!(Plano, "SELECT * FROM planos")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch Planos: {:?}", e);
            Html("<p>Failed to fetch Planos</p>".to_string())
        })
        .expect("Failed to fetch Planos");

    let template = ClienteEditTemplate {
        &client,
        mikrotik_options: mikrotik_list,
        plan_options: plan_list,
    }
    .render()
    .map_err(|e| -> _ {
        error!("Failed to render client edit template: {:?}", e);
        Html("<p>Failed to render client edit template</p>".to_string())
    })
    .expect("Failed to render client edit template");

    Html(template)
}
*/

//Gets the id of the cliente from the delete button
//Deletes the cliente from the db
//Returns a redirect of the user to the client list
pub async fn delete_cliente(
    Path(id): Path<i32>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    //TODO should delete it from radius aswell
    match delete_cliente_by_id(&pool, id).await {
        Ok(_) => Redirect::to("/cliente").into_response(),

        Err(e) => {
            error!("Failed to delete cliente: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//TODO this is not used because i dont have the edit form
//Gets the edited cliente from the form
//Updates the cliente in the db
//Returns a redirect of the user to the client list
pub async fn update_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(client): Form<Cliente>,
) -> impl IntoResponse {

    match update_cliente_by_id(&client, &pool).await {
        Ok(_) => Redirect::to("/cliente").into_response(),

        Err(e) => {
            error!("Failed to update cliente: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }

}

//Gets all the mikrotik options from the db
//Gets all the planos options from the db
//Renders the cliente form with the mikrotik and planos options to associate the cliente to
//Returns the form to the user
pub async fn show_cliente_form(
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let mikrotik_list = find_all_mikrotiks(&pool).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))
    .expect("Failed to fetch Mikrotiks"); 

    let plan_list = find_all_planos(&pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))
    .expect("Failed to fetch Planos"); 

    let mut context = tera::Context::new();
    context.insert("mikrotik_options", &mikrotik_list);
    context.insert("plan_options", &plan_list);

    match TEMPLATES.render("cliente/cliente_add.html", &context) {
        Ok(template) => Html(template).into_response(),

        //TODO deal with error
        Err(e) => {
            error!("Failed to render client form template: {:?}", e);
            let erro = format!("Erro ao renderizar formulario de cliente {:?}",e.kind);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
        }
    }
}

//Gets the client data from the form
//Validates the cpf or cnpj(Based on the TipoPessoa selected on the form),pessoa fisica validates cpf and juridica validates cnpj
//salva uma versao formatada e uma nao formatada do cpf/cnpj(Para uso em nota fiscal)
//salva a cliente para o sistema de gestao financeira asaas(facilita para gerar a assinatura do cliente)
//retorna um redirect do usuario para a lista de clientes
//this can be a source of errors on the cliente side pela falta de atencao
pub async fn register_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    State(state): State<AppState>,
    Form(mut client): Form<ClienteDto>,
) -> impl IntoResponse {
    //TODO extract this to a function
    //Validate the cpf/cnpj based on the Tipo de Pessoa
    //This is not really good,always forget to set the Tipo de Pessoa no formulario
    //TODO maybe we can set the Tipo de Pessoa based on the length of the cpf/cnpj on the frontend
    match client.tipo {
        TipoPessoa::PessoaFisica => {
            //Check the cpf
            if cpf::valid(&client.cpf_cnpj) {
                //Parse the cpf and save the formatted one to the db together with an unformated_one
                client.formatted_cpf_cnpj = client
                    .cpf_cnpj
                    .parse::<Cpf>()
                    .map_err(|e| {
                        error!("Failed to parse cpf/cnpj: {:?}", e);
                        return (axum::http::StatusCode::BAD_REQUEST,e.to_string());
                    })
                    .expect("Failed to parse cpf/cnpj")
                    .to_string();
            } else {
                return (axum::http::StatusCode::BAD_REQUEST,"CPF Invalido").into_response();
            }
        },

        TipoPessoa::PessoaJuridica => {
            //Check the cnpj(it looks kinda of buggy)
            //TODO make better checks for this shit
            //BUG this looks kinda of buggy
            //?maybe i could unit test?idk
            if cnpj::valid(&client.cpf_cnpj) {
                client.formatted_cpf_cnpj = client
                    .cpf_cnpj
                    .parse::<Cnpj>()
                    .map_err(|e| -> _ {
                        error!("Failed to parse cpf/cnpj: {:?}", e);
                        return (axum::http::StatusCode::BAD_REQUEST,e.to_string());
                    }).expect("Failed to parse cpf/cnpj")
                    .to_string();
            } else {
                return (axum::http::StatusCode::BAD_REQUEST,"CNPJ Invalido").into_response();
            }
        }
    }

    //Cria uma transacao para que nao se crie uma parte do cliente caso haja erro no futuro
    let transaction = pool.begin().await.expect("Erro ao iniciar transacao");

    save_cliente(&client, &pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao salvar cliente na db");

    let plano = find_plano_by_id(&pool, client.plano_id).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())   
    ).expect("Erro ao buscar plano pela id"); 

    //Cria cliente e assinatura no radius
    add_cliente_to_asaas(&client,&plano,&state.http_client).await.map_err(|e| {
        error!("Failed to add client to asaas: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string());   
    }).expect("Failed to add client to asaas");

    let cliente_radius = ClienteNas {
        username: client.login,
        password: client.senha,
        plano_nome: plano.nome,
    };

    add_cliente_radius(cliente_radius).await.map_err(|e| {
        error!("Failed to add client to radius: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string());
    }).expect("Failed to add client to radius");

    transaction.commit().await.expect("Erro ao commitar transacao");
    Redirect::to("/cliente").into_response()
}

// Templates

/* 
#[derive(Template)]
#[template(path = "cliente_edit.html")]
struct ClienteEditTemplate<'a> {
    client: &'a Cliente,
    mikrotik_options: Vec<Mikrotik>,
    plan_options: Vec<Plano>,
}

impl ClienteEditTemplate<'_> {
    fn is_pessoa_fisica(&self) -> bool {
        !self.client.tipo
    }

    fn is_pessoa_juridica(&self) -> bool {
        self.client.tipo
    }
}
*/




//TODO importar clientes do mkauth
//posso pegar o codigo do contractor,realizar auth,pegar clientes e adaptar para o meu formato de cliente
//adicionar para o asaas como opcao, seguir o gerar dici e nota fiscal setado pelo mkauth(criar radius com certeza)
//terei que ter uma segunda opcao em cliente
