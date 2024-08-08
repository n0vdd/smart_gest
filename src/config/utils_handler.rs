use std::env;
use axum::{extract::{Query, State}, response::{Html, IntoResponse}};
use cnpj::Cnpj;
use cpf::Cpf;
use phonenumber::country::Id::BR;
use serde::Deserialize;
use tera::Context;
use thiserror::Error;
use tracing::{debug, error};

use crate::{clientes::cliente_model::TipoPessoa, config::endereco_model::EnderecoDto, AppState, TEMPLATES};

//Recebe um cep do campo cep do formulario de endereco
//Envia um pedido para a api do webmania que retorna os dados do cep(rua,bairro,estado,cidade etc)
//Converte esses dados em um endereco
//renderiza uma template com os dados do endereco
//retorna um snippet do html de endereco com os dados prenchidos de acordo com a resposta da api
pub async fn lookup_cep(
    State(state): State<AppState>,
    Query(query): Query<CepQuery>,
) -> impl IntoResponse {
    debug!("Looking up CEP: {}", query.cep);

    // Retrieve the CEP API URL from environment variables
    let url_template = env::var("CEP_API_URL").map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("CEP_API_URL not set");

    let url = url_template.replace("%s", &query.cep);

    debug!("Requesting: {}", url);

    // Send the request to the CEP API
    let response = state.http_client.get(&url).send().await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())    
    ).expect("Erro ao fazer request para a api de cep");
    
    debug!("Response: {:?}", response);

    if response.status().is_success() {
        // Read the response body
        let response_body = response.text().await.map_err(|e|
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
        ).expect("Erro ao ler o corpo da resposta");

        // Deserialize the response body
        let webmania_response: WebmaniaResponse = serde_json::from_str(&response_body).map_err(|e|
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
        ).expect("Erro ao deserializar a resposta");

        let endereco = EnderecoDto {
            rua: webmania_response.endereco,
            numero: None,
            bairro: webmania_response.bairro,
            cidade: webmania_response.cidade,
            estado: webmania_response.uf,
            cep: webmania_response.cep,
            ibge: webmania_response.ibge,
            complemento: None,
        };
        debug!("Converted response to endereco: {:?}", endereco);

        // Render the template
        let mut context = tera::Context::new();
        context.insert("endereco", &endereco);

        match TEMPLATES.render("snippets/endereco_snippet.html", &context) {
            Ok(template) => Html(template).into_response(),

            Err(e) => {
                error!("Failed to render endereco snippet: {:?}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to render endereco snippet").into_response()
            }
        }
    } else {
        let err = format!("HTTP error: {:?}", CepError::CepNotFound);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
    }
}

//Mosta o snippet do endereco sem os dados
pub async fn show_endereco() -> impl IntoResponse {
    match TEMPLATES.render("snippets/endereco_snippet.html",&Context::new()) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render endereco snippet: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to render endereco snippet").into_response()
        }
    }
}

//Recebe um numero de telefone(assume que o mesmo vem do brasil), do campo telefone do cadastro de cliente
//Valida se o numero de telefone seria um numero de telefone brasileiro valido
//Caso o numero de telefone for valido, nao faz nada
//caso o numero seja invalido retorna um aviso no html 
pub async fn validate_phone(Query(phone): Query<TelefoneQuery>) -> impl IntoResponse {
    debug!("Validating phone number: {:?}", phone.telefone);

    let phone = phonenumber::parse(Some(BR), phone.telefone).map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Failed to parse phone number");

    if !phone.is_valid() {
        return (axum::http::StatusCode::BAD_REQUEST,"Numero de telefone invalido".to_string()).into_response()
    } else {
        Html("").into_response()
    }
}

//will use the tipo de pessoa for a check, if the user is a pf or pj
//Gets the cpf_cnpj from the cpf_cnpj field on the cliente form
//Gets the Tipo Pessoa from the tipo fiel on the cliente form aswell
//Based on the Tipo Pessoa it validates against cpf(Pessoa Fisica) or cnpj(Pessoa juridica)
//if it is a valid number it formats the data and return a html snippet with the formated cpf_cnpj on display,and the unformated one hidden on another elememt
pub async fn validate_cpf_cnpj(
    Query(cpf_cnpj): Query<CpfCnpjQuery>,
) -> impl IntoResponse {
    debug!("Validating CPF/CNPJ: {:?}", cpf_cnpj.formatted_cpf_cnpj);
    let mut context = tera::Context::new();
    context.insert("cpf_cnpj", &cpf_cnpj.formatted_cpf_cnpj);

    match cpf_cnpj.tipo {
        TipoPessoa::PessoaFisica => {
            match cpf_cnpj.formatted_cpf_cnpj.parse::<Cpf>() {
                Ok(cpf) => {
                    let formatted = cpf.to_string();
                    context.insert("formatted_cpf_cnpj", &formatted);
            
                    match TEMPLATES.render("snippets/cpf_cnpj_snippet.html", &context) {
                        Ok(template) => Html(template).into_response(),
                        Err(e) => {
                            error!("Failed to render CPF/CNPJ snippet: {:?}", e);
                            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to render CPF/CNPJ snippet").into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse CPF: {:?}", e);
                    (axum::http::StatusCode::BAD_REQUEST, "Failed to parse CPF".to_string()).into_response()
                }
            }
        },
        TipoPessoa::PessoaJuridica => {
            match cpf_cnpj.formatted_cpf_cnpj.parse::<Cnpj>() {
                Ok(cnpj) => {
                    let formatted = cnpj.to_string();
                    context.insert("formatted_cpf_cnpj", &formatted);
            
                    match TEMPLATES.render("snippets/cpf_cnpj_snippet.html", &context) {
                        Ok(template) => Html(template).into_response(),
                        Err(e) => {
                            error!("Failed to render CPF/CNPJ snippet: {:?}", e);
                            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to render CPF/CNPJ snippet").into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse CNPJ: {:?}", e);
                    (axum::http::StatusCode::BAD_REQUEST, "Failed to parse CNPJ".to_string()).into_response()
                }
            }
        }
    }
}


#[derive(Deserialize)]
pub struct CpfCnpjQuery {
    formatted_cpf_cnpj: String,
    tipo: TipoPessoa,
}

#[derive(Debug, Deserialize)]
struct WebmaniaResponse {
    endereco: String,
    bairro: String,
    cidade: String,
    uf: String,
    cep: String,
    ibge: String,
}

#[derive(Error, Debug)]
pub enum CepError {
    #[error("HTTP request failed")]
    HttpRequestFailed(#[from] reqwest::Error),
    #[error("Failed to deserialize response")]
    DeserializationFailed(#[from] serde_json::Error),
    #[error("CEP not found")]
    CepNotFound,
}

#[derive(Deserialize)]
pub struct TelefoneQuery {
    telefone: String,
}

#[derive(Deserialize)]
pub struct CepQuery {
    cep: String,
}

