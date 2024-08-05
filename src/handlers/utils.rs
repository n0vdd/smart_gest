use std::env;
use axum::{extract::Query, response::Html};
use cnpj::Cnpj;
use cpf::Cpf;
use phonenumber::country::Id::BR;
use reqwest::Client;
use serde::Deserialize;
use tera::Context;
use thiserror::Error;
use tracing::{debug, error};

use crate::{models::{client::TipoPessoa, endereco::EnderecoDto}, TEMPLATES};



//Recebe um cep do campo cep do formulario de endereco
//Envia um pedido para a api do webmania que retorna os dados do cep(rua,bairro,estado,cidade etc)
//Converte esses dados em um endereco
//renderiza uma template com os dados do endereco
//retorna um snippet do html de endereco com os dados prenchidos de acordo com a resposta da api
pub async fn lookup_cep(
    Query(query): Query<CepQuery>,
) -> Result<Html<String>, Html<String>> {
    debug!("Looking up CEP: {}", query.cep);

    let url_template = env::var("CEP_API_URL")
        .map_err(|e | -> _ {
            error!("Erro:{:?} ao pegar variavel do ambiente: CEP_API_URL", e);
            return Html("Erro ao pegar variavel do ambiente: CEP_API_URL".to_string())
        })?;

    let url = url_template.replace("%s", &query.cep);

    debug!("Requesting: {}", url);
    let client = Client::new();
    let response = client.get(&url).send().await
    .map_err(|e | -> _ {
        error!("Failed to send request: {:?}", e);
        return Html("Falha ao enviar pedido para a API".to_string())
    })?;

    debug!("Response: {:?}", response);

    if response.status().is_success() {
        let response_body = response.text().await
        .map_err(|e | -> _ {
            error!("Failed to get response body: {:?}", e);
            return Html("Falha para recuperar a resposta da API".to_string())
        })?;
        let webmania_response: WebmaniaResponse = serde_json::from_str(&response_body)
        .map_err(|e| -> _{
            error!("Failed to deserialize response: {:?}", e);
            return Html("Falha ao realizar parse da resposta json da api".to_string())
        })?;
        let endereco = EnderecoDto {
            rua: webmania_response.endereco,
            numero: None,
            bairro: webmania_response.bairro,
            cidade: webmania_response.cidade,
            estado: webmania_response.uf,
            cep:  webmania_response.cep,
            ibge: webmania_response.ibge,
            complemento: None,
        };

        debug!("converted response to endereco: {:?}", endereco);

        let mut context = tera::Context::new();
        context.insert("endereco", &endereco);

        //TODO alterar o endereco snippet para usar endereco.cep etc
        let template = TEMPLATES.render("snippets/endereco_snippet.html", &context).map_err(|e| -> _ {
            error!("Failed to render endereco snippet: {:?}", e);
            return Html("Falha ao renderizar endereco snippet".to_string())
        })?;

        Ok(Html(template))
    } else {
        let err = format!("HTTP error: {:?}", CepError::CepNotFound);
        Err(Html(err))
    }
}



//Mosta o snippet do endereco sem os dados
pub async fn show_endereco() -> Html<String> {

    let template = TEMPLATES.render("snippets/endereco_snippet.html",&Context::new()).map_err(|e| -> _ {
        error!("Failed to render endereco snippet: {:?}", e);
        return Html("Failed to render endereco snippet".to_string())
    }).expect("Failed to render endereco snippet");

    Html(template)
}

//Recebe um numero de telefone(assume que o mesmo vem do brasil), do campo telefone do cadastro de cliente
//Valida se o numero de telefone seria um numero de telefone brasileiro valido
//Caso o numero de telefone for valido, nao faz nada
//caso o numero seja invalido retorna um aviso no html 
//TODO creta better error displaying on the frontend
pub async fn validate_phone(Query(phone): Query<TelefoneQuery>) -> Html<String> {
    debug!("Validating phone number: {:?}", phone.telefone);
    let phone = phonenumber::parse(Some(BR), phone.telefone).map_err(
        |e| -> _ {
            error!("Failed to parse phone number: {:?}", e);
            return Html("Falha para validar o numero de telefone".to_string())
        }
    ).expect("Failed to parse phone number");
    if !phone.is_valid() {
        Html("Numero de telefone invalido".to_string())
    } else {
        Html("".to_string())
    }
}

//will use the tipo de pessoa for a check, if the user is a pf or pj
//Gets the cpf_cnpj from the cpf_cnpj field on the cliente form
//Gets the Tipo Pessoa from the tipo fiel on the cliente form aswell
//Based on the Tipo Pessoa it validates against cpf(Pessoa Fisica) or cnpj(Pessoa juridica)
//if it is a valid number it formats the data and return a html snippet with the formated cpf_cnpj on display,and the unformated one hidden on another elememt
pub async fn validate_cpf_cnpj(
    Query(cpf_cnpj): Query<CpfCnpjQuery>,
) -> Html<String> {
    debug!("Validating CPF/CNPJ: {:?}", cpf_cnpj.formatted_cpf_cnpj);
    let mut context = tera::Context::new();
    context.insert("cpf_cnpj", &cpf_cnpj.formatted_cpf_cnpj);

    match cpf_cnpj.tipo {
        TipoPessoa::PessoaFisica => {
            let formatted = cpf_cnpj.formatted_cpf_cnpj.parse::<Cpf>().map_err(|e| -> _
            {
                error!("Failed to parse CPF: {:?}", e);
                return Html("Failed to parse CPF".to_string())
            })
            .expect("Failed to parse CPF").to_string();

            context.insert("formatted_cpf_cnpj", &formatted);

            let template = TEMPLATES.render("snippets/cpf_cnpj_snippet.html", &context).map_err(|e| -> _ {
                error!("Failed to render CPF/CNPJ snippet: {:?}", e);
                return Html("Failed to render CPF/CNPJ snippet".to_string())
            }).expect("Failed to render CPF/CNPJ snippet");

            Html(template)
        },
        TipoPessoa::PessoaJuridica => {
            let formatted = cpf_cnpj.formatted_cpf_cnpj.parse::<Cnpj>().map_err(|e| -> _
            {
                error!("Failed to parse CNPJ: {:?}", e);
                return Html("Failed to parse CNPJ".to_string())
            })
            .expect("Failed to parse CNPJ").to_string();

            context.insert("formatted_cpf_cnpj", &formatted);

            let template = TEMPLATES.render("snippets/cpf_cnpj_snippet.html", &context).map_err(|e| -> _ {
                error!("Failed to render CPF/CNPJ snippet: {:?}", e);
                return Html("Failed to render CPF/CNPJ snippet".to_string())
            }).expect("Failed to render CPF/CNPJ snippet");

            Html(template)
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

