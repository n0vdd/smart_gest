use std::{env, sync::Arc};

use axum::{extract::Query, Extension, Json};
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{Cep, Endereco};

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

#[derive(Clone)]
pub struct CepService {
    client: Client,
    url_template: String,
}

impl CepService {
    pub fn new() -> Self {
        let client = Client::new();
        let url_template = env::var("CEP_API_URL").expect("CEP_API_URL must be set");
        Self { client, url_template }
    }

    pub async fn consultar(&self, cep: &str) -> Result<Endereco, CepError> {
        //TODO look at how it expects the url
        let url = self.url_template.replace("{%s}", cep);
        debug!("Requesting: {}", url);
        let response = self.client.get(&url).send().await?;
        debug!("Response: {:?}", response);
        if response.status().is_success() {
            let response_body = response.text().await?;
            let webmania_response: WebmaniaResponse = serde_json::from_str(&response_body)?;
            let endereco = Endereco {
                endereco: webmania_response.endereco,
                bairro: webmania_response.bairro,
                cidade: webmania_response.cidade,
                estado: webmania_response.uf,
                cep: Cep { cep: webmania_response.cep },
                ibge: webmania_response.ibge,
                complemento: None,
            };
            debug!("converted response to endereco: {:?}",endereco);
            Ok(endereco)
        } else {
            Err(CepError::CepNotFound)
        }
    }
}

#[derive(Deserialize)]
pub struct CepQuery {
    cep: String,
}

//TODO, this should be returning HTML for htmx to use
pub async fn lookup_cep(
    Query(query): Query<CepQuery>,
    Extension(cep_service): Extension<Arc<CepService>>,
) -> Json<Result<Endereco, String>> {
    debug!("Looking up CEP: {}", query.cep);
    match cep_service.consultar(&query.cep).await {
        Ok(endereco) => Json(Ok(endereco)),
        Err(err) => Json(Err(format!("Failed to fetch address: {}", err))),
    }
}
