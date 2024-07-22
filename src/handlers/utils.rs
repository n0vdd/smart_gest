use std::env;
use askama::Template;
use axum::{extract::Query, response::Html};
use cnpj::Cnpj;
use cpf::Cpf;
use phonenumber::country::Id::BR;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, error};

use crate::handlers::clients::EnderecoDto;

use super::clients::TipoPessoa;
pub async fn lookup_cep(
    Query(query): Query<CepQuery>,
) -> Result<Html<String>, Html<String>> {
    debug!("Looking up CEP: {}", query.cep);

    let url_template = env::var("CEP_API_URL")
        .map_err(|e | -> _ {
            error!("Erro:{:?} ao pegar variavel do ambiente: CEP_API_URL", e);
            Html("Erro ao pegar variavel do ambiente: CEP_API_URL".to_string())
        })?;

    let url = url_template.replace("%s", &query.cep);

    debug!("Requesting: {}", url);
    let client = Client::new();
    let response = client.get(&url).send().await
    .map_err(|e | -> _ {
        error!("Failed to send request: {:?}", e);
        Html("Failed to send request".to_string())
    })?;

    debug!("Response: {:?}", response);

    if response.status().is_success() {
        let response_body = response.text().await
        .map_err(|e | -> _ {
            error!("Failed to get response body: {:?}", e);
            Html("Failed to get response body".to_string())
        })?;
        let webmania_response: WebmaniaResponse = serde_json::from_str(&response_body)
        .map_err(|e| -> _{
            error!("Failed to deserialize response: {:?}", e);
            Html("Failed to deserialize response".to_string())
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

        let template = EnderecoSnippetTemplate {
            cep: endereco.cep,
            rua: endereco.rua,
            bairro: endereco.bairro,
            cidade: endereco.cidade,
            estado: endereco.estado,
            ibge: endereco.ibge,
        };

        let template = template.render().map_err(|e| -> _ {
            error!("Failed to render endereco snippet: {:?}", e);
            Html("Failed to render endereco snippet".to_string())
        })?;
        Ok(Html(template))
    } else {
        let err = format!("HTTP error: {:?}", CepError::CepNotFound);
        Err(Html(err))
    }
}


#[derive(Template)]
#[template(path = "snippets/cpf_cnpj_snippet.html")]
pub struct CpfCnpjTemplate {
    pub formatted_cpf_cnpj: String,
    pub cpf_cnpj: String,
}

#[derive(Template)]
#[template(path = "snippets/telefone_snippet.html")]
pub struct TelefoneTemplate {
    pub telefone: String,
}

pub async fn show_endereco() -> Html<String> {
    let template = EnderecoSnippetTemplate {
        cep: "".to_string(),
        rua: "".to_string(),
        bairro: "".to_string(),
        cidade: "".to_string(),
        estado: "".to_string(),
        ibge: "".to_string(),
    }.render().map_err(|e| -> _ {
        error!("Erro ao renderizar endereco snippet: {:?}", e);
        Html("Erro ao renderizar endereco snippet".to_string())
    }).expect("Erro ao renderizar endereco snippet");

    Html(template)
}

//TODO validate phonumber(will deal with just brazilian numbers)
pub async fn validate_phone(Query(phone): Query<TelefoneQuery>) -> Html<String> {
    debug!("Validating phonelfe number: {:?}", phone.telefone);
    //TODO check if the phone number is valid
    let phone = phonenumber::parse(Some(BR), phone.telefone).map_err(
        |e| -> _ {
            error!("Failed to parse phone number: {:?}", e);
            return Html("Failed to parse phone number".to_string())
        }
    ).expect("Failed to parse phone number");
    if !phone.is_valid() {
        Html("Invalid phone number".to_string())
    } else {
        //TODO maybe there is a better way to do this
        Html("".to_string())
    }
}

//TODO this will be called by htmx after the user inputs the cpf/cnpj
//will use the tipo de pessoa for a check, if the user is a pf or pj
pub async fn validate_cpf_cnpj(
    Query(cpf_cnpj): Query<CpfCnpjQuery>,
) -> Html<String> {
    debug!("Validating CPF/CNPJ: {:?}", cpf_cnpj.formatted_cpf_cnpj);
    match cpf_cnpj.tipo {
        TipoPessoa::PessoaFisica => {
            let formatted = cpf_cnpj.formatted_cpf_cnpj.parse::<Cpf>().map_err(|e| -> _
            {
                error!("Failed to parse CPF: {:?}", e);
                Html("Failed to parse CPF".to_string())
            })
            .expect("Failed to parse CPF").to_string();

            let template = CpfCnpjTemplate {
                formatted_cpf_cnpj: formatted,
                cpf_cnpj: cpf_cnpj.formatted_cpf_cnpj,
            }.render().map_err(|e| -> _ {
                error!("Failed to render CPF/CNPJ snippet: {:?}", e);
                Html("Failed to render CPF/CNPJ snippet".to_string())
            }).expect("Failed to render CPF/CNPJ snippet");

            Html(template)
        },
        TipoPessoa::PessoaJuridica => {
            let formatted = cpf_cnpj.formatted_cpf_cnpj.parse::<Cnpj>().map_err(|e| -> _
            {
                error!("Failed to parse CNPJ: {:?}", e);
                Html("Failed to parse CNPJ".to_string())
            })
            .expect("Failed to parse CNPJ").to_string();

            let template = CpfCnpjTemplate {
                formatted_cpf_cnpj: formatted,
                cpf_cnpj: cpf_cnpj.formatted_cpf_cnpj,
            }.render().map_err(|e| -> _ {
                error!("Failed to render CPF/CNPJ snippet: {:?}", e);
                Html("Failed to render CPF/CNPJ snippet".to_string())
            }).expect("Failed to render CPF/CNPJ snippet");

            Html(template)
        }
    }
}

//TODO gerar dici, tenho a template de como deve ser
//a formatacao que precisa ser usada e etc,vou precisar da crate de csv


#[derive(Template)]
#[template(path = "snippets/endereco_snippet.html")]
struct EnderecoSnippetTemplate{
    cep: String,
    rua: String,
    bairro: String,
    cidade: String,
    estado: String,
    ibge: String,
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

