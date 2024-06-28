use std::env;
use askama::Template;
use axum::{extract::Query, response::Html};
use cnpj::Cnpj;
use cpf::Cpf;
use log::{debug, error};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

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


//TODO this will be called by htmx after the user inputs the cpf/cnpj
//will use the tipo de pessoa for a check, if the user is a pf or pj
pub async fn validate_cpf_cnpj(
    Query(cpf_cnpj): Query<CpfCnpjQuery>,
) -> Html<String> {
    match cpf_cnpj.tipo {
        TipoPessoa::PessoaFisica => {
            let formatted = cpf_cnpj.cpf_cnpj.parse::<Cpf>().map_err(|e| -> _
            {
                error!("Failed to parse CPF: {:?}", e);
                Html("Failed to parse CPF".to_string())
            })
            .expect("Failed to parse CPF").to_string();

            return Html(format!(
                r#"
                <div class="mb-4">
                    <label for="cpf_cnpj" class="block text-gray-700 text-sm font-bold mb-2">CPF/CNPJ:</label>
                    <input type="text" id="cpf_cnpj" name="cpf_cnpj" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
                </div>
                "#,
                formatted
            ));
        }
        TipoPessoa::PessoaJuridica => {
            let formatted = cpf_cnpj.cpf_cnpj.parse::<Cnpj>().map_err(|e| -> _
            {
                error!("Failed to parse CNPJ: {:?}", e);
                Html("Failed to parse CNPJ".to_string())
            })
            .expect("Failed to parse CNPJ").to_string();

            //TODO this will be called by htmx after the user inputs the cpf/cnpj
            //use askama and create the snippet
            return Html(format!(
                r#"
                <div class="mb-4">
                    <label for="cpf_cnpj" class="block text-gray-700 text-sm font-bold mb-2">CPF/CNPJ:</label>
                    <input type="text" id="cpf_cnpj" name="cpf_cnpj" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
                </div>
                "#,
                formatted
            ));       
        }
    }
}

#[derive(Template)]
#[template(path = "endereco_snippet.html")]
struct EnderecoSnippetTemplate{
    rua: String,
    bairro: String,
    cidade: String,
    estado: String,
    ibge: String,
}

#[derive(Deserialize)]
pub struct CpfCnpjQuery {
    cpf_cnpj: String,
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
pub struct CepQuery {
    #[serde(rename = "endereco.cep")]
    cep: String,
}

