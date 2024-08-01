use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use radius::{bloqueia_cliente_radius, add_cliente_radius, ClienteNas};
use tera::Tera;
use time::{macros::format_description, PrimitiveDateTime};
use tracing::{debug, error};
use axum_extra::extract::Form;
use cnpj::Cnpj;
use cpf::Cpf;
use std::sync::Arc;
use sqlx::{query, query_as, PgPool};


use crate::{models::{client::{Cliente, ClienteDto, SimpleCliente, TipoPessoa}, mikrotik::Mikrotik, plano::Plano}, services::webhooks::add_cliente_to_asaas, TEMPLATES};

async fn get_all_clientes(pool: &PgPool) -> Result<Vec<SimpleCliente>, anyhow::Error> {
    query_as!(SimpleCliente, "SELECT id,login FROM clientes")
        .fetch_all(pool)
        .await.map_err(|e|{
            error!("Failed to fetch clients: {:?}", e);
            anyhow::anyhow!("Failed to fetch all clients")
        })
}


//Get all the clientes from the db
//Render the template with the clientes
//return the client list
pub async fn show_cliente_list(
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let clients = query_as!(Cliente, "SELECT * FROM clientes")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to fetch clients: {:?}", e);
            return Html("<p>Failed to fetch clients</p>".to_string())
        }).expect("Failed to fetch clients");


    let mut context = tera::Context::new();
    context.insert("clients", &clients);

    let template = TEMPLATES.render("cliente_list.html", &context).map_err(|e| -> _ {
        error!("Failed to render client list template: {:?}", e);
        return Html("<p>Failed to render client list template</p>".to_string())
    }).expect("Failed to render client list template");

    Html(template)
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

    query!("DELETE FROM clientes WHERE id = $1", id)
        .execute(&*pool)
        .await
        .map_err(|e| -> _ {
            error!("Failed to delete client: {:?}", e);
            return Html("<p>Failed to delete client</p>".to_string())
        }).expect("Failed to delete client");

    Redirect::to("/cliente").into_response()
}

//TODO this is not used because i dont have the edit form
//Gets the edited cliente from the form
//Updates the cliente in the db
//Returns a redirect of the user to the client list
pub async fn update_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(client): Form<Cliente>,
) -> impl IntoResponse {

    query!(
        "UPDATE clientes SET tipo = $1, nome = $2, email = $3, cpf_cnpj = $4, formatted_cpf_cnpj = $5,
        telefone = $6, login = $7, senha = $8, cep = $9, rua = $10, numero = $11, bairro = $12,
        complemento = $13, cidade = $14, estado = $15, ibge_code = $16, mikrotik_id = $17,
        plano_id = $18 WHERE id = $19",
        client.tipo,
        client.nome,
        client.email,
        client.cpf_cnpj,
        client.formatted_cpf_cnpj,
        client.telefone,
        client.login,
        client.senha,
        client.cep,
        client.rua,
        client.numero,
        client.bairro,
        client.complemento,
        client.cidade,
        client.estado,
        client.ibge_code,
        client.mikrotik_id,
        client.plano_id,
        client.id
    )
    .execute(&*pool)
    .await
    .map_err(|e| -> _ {
        error!("Failed to update client: {:?}", e);
        return Html("<p>Failed to update client</p>".to_string())
    }).expect("Failed to update client");

    Redirect::to("/cliente").into_response()
}

//Gets all the mikrotik options from the db
//Gets all the planos options from the db
//Renders the cliente form with the mikrotik and planos options to associate the cliente to
//Returns the form to the user
pub async fn show_cliente_form(
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let mikrotik_list = query_as!(Mikrotik, "SELECT * FROM mikrotik")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            debug!("Failed to fetch Mikrotik: {:?}", e);
            return Html("<p>Failed to fetch all Mikrotiks</p>".to_string())
        }).expect("error fetching mikrotik");

    let plan_list = query_as!(Plano, "SELECT * FROM planos")
        .fetch_all(&*pool)
        .await
        .map_err(|e| -> _ {
            debug!("Failed to fetch Planos: {:?}", e);
            Html("<p>Failed to fetch all Planos</p>".to_string())
        }).expect("Failed to fetch Planos");

    let mut context = tera::Context::new();
    context.insert("mikrotik_options", &mikrotik_list);
    context.insert("plan_options", &plan_list);

    let template = TEMPLATES.render("cliente_add.html", &context).map_err(|e| -> _ {
        error!("Failed to render client form template: {:?}", e);
        return Html("<p>Failed to render client form template</p>".to_string())
    }).expect("Failed to render client form template");

    Html(template)
}

//Gets the client data from the form
//Validates the cpf or cnpj(Based on the TipoPessoa selected on the form),pessoa fisica validates cpf and juridica validates cnpj
//salva uma versao formatada e uma nao formatada do cpf/cnpj(Para uso em nota fiscal)
//salva a cliente para o sistema de gestao financeira asaas(facilita para gerar a assinatura do cliente)
//retorna um redirect do usuario para a lista de clientes
//this can be a source of errors on the cliente side pela falta de atencao
pub async fn register_cliente(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(mut client): Form<ClienteDto>,
) -> impl IntoResponse {

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
                    .map_err(|e| -> _ {
                        error!("Failed to parse cpf/cnpj: {:?}", e);
                        //I can return the Response to the fronted from the error itself
                        //TODO this should appear bellow the cpf/cnpj field with the wrong data on it, not on another page
                        return Html("<p>Falha ao formatar Cpf</p>".to_string())
                    })
                    .expect("Failed to parse cpf/cnpj")
                    .to_string();
            } else {
                //TODO this should appear bellow the cpf/cnpj field with the wrong data on it, not on another page
                return Html("<p>CPF Invalido</p>".to_string()).into_response();
            }
        },

        TipoPessoa::PessoaJuridica => {
            //Check the cnpj(it looks kinda of buggy)
            //TODO make better checks for this shit
            //?maybe i could unit test?idk
            if cnpj::valid(&client.cpf_cnpj) {
                client.formatted_cpf_cnpj = client
                    .cpf_cnpj
                    .parse::<Cnpj>()
                    .map_err(|e| -> _ {
                        error!("Failed to parse cpf/cnpj: {:?}", e);
                        //TODO this should appear bellow the cpf/cnpj field with the wrong data on it, not on another page
                        return Html("<p>Erro ao formatar o Cnpj</p>".to_string())
                    }).expect("Failed to parse cpf/cnpj")
                    .to_string();
            } else {
                //TODO this should appear bellow the cpf/cnpj field with the wrong data on it, not on another page
                return Html("<p>CNPJ Invalido</p>".to_string()).into_response();
            }
        }
    }

    //Apos formatar o cpf/cnpj salva o cliente para a db(endereco faz parte do mesmo), poderia separar,
    //mas nao vejo necessidade nesse caso,caso da vedajato sera necessario
    query!(
        "INSERT INTO clientes (
            tipo, nome, email, cpf_cnpj, formatted_cpf_cnpj, telefone, login, senha, 
            mikrotik_id, plano_id, cep, rua, numero, bairro, complemento, cidade, estado, ibge_code
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        )",
        client.tipo.as_bool(),
        client.nome,
        client.email,
        //cpf_cnpj nao formatado sera usado para nota_fiscal
        client.cpf_cnpj,
        //cpf_cnpj formatado sera usado para exibir na pagina e para o contrato
        client.formatted_cpf_cnpj,
        client.telefone,
        //login e senha usado para controle de acesso pelo radius
        client.login,
        client.senha,
        client.mikrotik_id,
        client.plano_id,
        client.endereco.cep,
        client.endereco.rua,
        client.endereco.numero,
        client.endereco.bairro,
        client.endereco.complemento,
        client.endereco.cidade,
        client.endereco.estado,
        client.endereco.ibge
    )
    .execute(&*pool).await.map_err(|e| {
        error!("Failed to insert client: {:?}", e);
        return Html("Failed to save the client")
    }).expect("Failed to insert client");

    let plano= query_as!(Plano,"SELECT * FROM planos WHERE id = $1", client.plano_id)
    .fetch_one(&*pool).await
    .map_err(|e| {
        error!("Failed to fetch plan name: {:?}", e);
        return Html("Failed to fetch plan name")
    }).expect("Failed to fetch plan name");

    //Cria cliente e assinatura no radius
    add_cliente_to_asaas(&client,&plano).await.map_err(|e| {
        error!("Failed to add client to asaas: {:?}", e);
        return Html("Failed to add client to asaas")
    }).expect("Failed to add client to asaas");

    let cliente_radius = ClienteNas {
        username: client.login,
        password: client.senha,
        plano_nome: plano.nome,
    };

    add_cliente_radius(cliente_radius).await.map_err(|e| {
        error!("Failed to add client to radius: {:?}", e);
        return Html("Failed to add client to radius")
    }).expect("Failed to add client to radius");

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


//param: date, a date to be compared on the db with the time the cliente was created
//Selects the tipo of the cliente(Pessoa Fisica ou Juridica) for all the clientes created before the given date
//Returns a List with all the tipos of the clientes created before the given date
pub async fn fetch_tipo_clientes_before_date(
    pool: &PgPool,
    date: PrimitiveDateTime
) -> Result<Vec<TipoPessoa>,anyhow::Error> {
    //Get the tipo of the cliente created before a given data
    let tipos = query!("SELECT tipo FROM clientes WHERE created_at < $1", date)
        .fetch_all(&*pool)
        .await.map_err(|e| -> _ {
            error!("Failed to fetch clients: {:?}", e);
            anyhow::anyhow!("Falha ao achar clientes na db antes da data {date}")
        })?
        //Convert the bool from the record to TipoPessoa
        .iter().map(|row| {
            TipoPessoa::from_bool(row.tipo)
        }).collect();

    Ok(tipos)
}

pub async fn bloqueia_clientes_atrasados(pool: &PgPool) -> Result<(),anyhow::Error>{
    let clientes = get_all_clientes(&*pool).await.map_err(|e| {
        error!("Failed to fetch clientes: {:?}", e);
        anyhow::anyhow!("Failed to fetch all clientes")
    }).expect("Erro ao buscar clientes");

    for cliente in clientes {
        //Format chrono to primitiveDateTive
        let format = format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
        let date = PrimitiveDateTime::parse(chrono::Utc::now().to_string().as_str(), format).expect("Erro ao formatar data");

        //BUG maybe the way i am cheking the date is not the best
        let payment = query!("SELECT * FROM payment_confirmed where cliente_id = $1 and created_at >= $2 and created_at <= $3",cliente.id,date,date)
            .fetch_optional(&*pool).await.map_err(|e| {
                error!("Failed to fetch payment_confirmed: {:?}", e);
                anyhow::anyhow!("Failed to fetch all payment_confirmed")
        }).expect("Erro ao buscar payment_confirmed");

        if payment.is_none() {
            bloqueia_cliente_radius(&cliente.login).await.map_err(|e| {
                error!("Failed to block cliente: {:?}", e);
                anyhow::anyhow!("Failed to block cliente")
            }).expect("Erro ao bloquear cliente");
        }
    }

    Ok(())
}