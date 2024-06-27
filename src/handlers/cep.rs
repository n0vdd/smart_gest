use std::env;
use axum::{extract::Query, response::Html};
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use serde_json::to_string;
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

#[derive(Deserialize)]
pub struct CepQuery {
    #[serde(rename = "endereco.cep")]
    cep: String,
}

pub async fn lookup_cep(
    Query(query): Query<CepQuery>,
) -> Result<Html<String>, Html<String>> {
    debug!("Looking up CEP: {}", query.cep);
    let url_template = env::var("CEP_API_URL").unwrap();
    let url = url_template.replace("%s", &query.cep);
    debug!("Requesting: {}", url);
    let client = Client::new();
    let response = client.get(&url).send().await.map_err(|e| Html(format!("Error: {:?}", e)))?;
    debug!("Response: {:?}", response);
    if response.status().is_success() {
        let response_body = response.text().await.map_err(|e| Html(format!("Error: {:?}", e)))?;
        let webmania_response: WebmaniaResponse = serde_json::from_str(&response_body).map_err(|e| Html(format!("Error: {:?}", e)))?;
        let endereco = Endereco {
            rua: webmania_response.endereco,
            numero: None,
            bairro: webmania_response.bairro,
            cidade: webmania_response.cidade,
            estado: webmania_response.uf,
            cep: Cep { cep: webmania_response.cep },
            ibge: webmania_response.ibge,
            complemento: None,
        };
        debug!("converted response to endereco: {:?}", endereco);
        let html = format!(
            r#"
            <div class="mb-4">
                <label for="rua" class="block text-gray-700 text-sm font-bold mb-2">Rua:</label>
                <input type="text" id="rua" name="endereco.rua" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            <div class="mb-4">
                <label for="numero" class="block text-gray-700 text-sm font-bold mb-2">Número:</label>
                <input type="text" id="numero" name="endereco.numero" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            <div class="mb-4">
                <label for="bairro" class="block text-gray-700 text-sm font-bold mb-2">Bairro:</label>
                <input type="text" id="bairro" name="endereco.bairro" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            <div class="mb-4">
                <label for="complemento" class="block text-gray-700 text-sm font-bold mb-2">Complemento:</label>
                <input type="text" id="complemento" name="endereco.complemento" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            <div class="mb-4">
                <label for="cidade" class="block text-gray-700 text-sm font-bold mb-2">Cidade:</label>
                <input type="text" id="cidade" name="endereco.cidade" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            <div class="mb-4">
                <label for="estado" class="block text-gray-700 text-sm font-bold mb-2">Estado:</label>
                <input type="text" id="estado" name="endereco.estado" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            <div class="mb-4">
                <label for="ibge" class="block text-gray-700 text-sm font-bold mb-2">IBGE:</label>
                <input type="text" id="ibge" name="endereco.ibge" value="{}" class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline">
            </div>
            "#,
            endereco.rua, endereco.bairro, endereco.cidade, endereco.estado, endereco.ibge
        );
        Ok(Html(html))
    } else {
        let err = format!("HTTP error: {:?}", CepError::CepNotFound);
        Err(Html(err))
    }
}
