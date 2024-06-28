//TODO one handler will show a form for uploading the contract html with its name

use askama::Template;
use serde::{Deserialize, Serialize};

use super::clients::Endereco;

//TODO criar modelo de dados usados para gerar o contrato
//terei que buscar como um subset dos dados do cliente(posso pegar pela db)
#[derive(Template)]
#[template(path = "contrato_padrao_fibra.html")]
pub struct ContratoPadraoFibra {
    client: ClienteContractData,
    data: String
}

#[derive(Template)]
#[template(path = "contrato_padrao_fibra+voip.html")]
pub struct ContratoPadraoFibraVoip {
    client: ClienteContractData,
    data: String
}

#[derive(Template)]
#[template(path = "contrato_padrao_voip.html")]
pub struct ContratoPadraoVoip {
    client: ClienteContractData,
    data: String,
}
/// Struct for storing formatted client data.
#[derive(Serialize, Deserialize, Debug)]
struct ClienteContractData {
    id: String,
    nome: String,
    login: String,
    endereco: Endereco,
    formatted_cpf_cnpj: String,
}


/*TODO talvez preicise ter algo diferente para o contrato gerado
//ter um campo com os templates e um campo para os gerados(linkar com o cliente)
//Sera os html que o sistema aceita upload
//ele salva em alguma pasta e linka o caminho
#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct ContratoTemplate {
    pub nome: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
//serao salvos os contratos relacionados com os clientes
//salva em alguma pasta e linka o caminho
pub struct Contrato {
    pub nome: String,
    pub path: String,
}
   */ 
//the other will be for generating the contract
/*based on this code
use anyhow::Result;
use askama::Template;
use axum::{response::{Html, Redirect}, routing::{get, post}, Router};
use axum_extra::extract::Form;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Local;
use cnpj::Cnpj;
use cpf::Cpf;
use env_logger;
use log::{debug, error, info};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions}, io::Write, net::SocketAddr, process::Command
};
use tokio::net::TcpListener;

/// URL for authentication API.
const AUTH_URL: &str = "https://172.27.27.27/api/auth";

/// Client ID for authentication.
const CLIENT_ID: &str = "Client_Id_21232f297a57a5a743894a0e4a801fc3";

/// Client secret for authentication.
const CLIENT_SECRET: &str = "Client_Secret_254f4ac462b2e5ff7eb5b952f89ab79f550b89e9";

/// URL for client list API.
const LIST_API_URL: &str = "https://172.27.27.27/api/cliente/listagem";

/// File path for storing client data in JSON format.
const CLIENTS_JSON_FILE: &str = "clientes.json";

/// File path for temporary HTML output.
const OUTPUT_HTML_FILE: &str = "contrato_temp.html";

/// Template struct for the index page.
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    clients: &'a [ClientData],
}

/// Template struct for the fiber contract.
#[derive(Template)]
#[template(path = "contrato_padrao_fibra.html")]
struct ContratoFibraTemplate<'a> {
    client: &'a ClientData,
    data: String,
}

/// Template struct for the VoIP contract.
#[derive(Template)]
#[template(path = "contrato_padrao_voip.html")]
struct ContratoVoipTemplate<'a> {
    client: &'a ClientData,
    data: String,
}

/// Template struct for the fiber + VoIP contract.
#[derive(Template)]
#[template(path = "contrato_padrao_fibra+voip.html")]
struct ContratoFibraVoipTemplate<'a> {
    client: &'a ClientData,
    data: String,
}

/// Struct for client data received from the API.
#[derive(Serialize, Deserialize, Debug)]
struct ClientDataJson {
    uuid: String,
    id: String,
    nome: String,
    login: String,
    endereco: Option<String>,
    numero: Option<String>,
    bairro: Option<String>,
    complemento: Option<String>,
    cidade: Option<String>,
    cep: Option<String>,
    estado: Option<String>,
    cpf_cnpj: String,
}

/// Struct for the list of clients received from the API.
#[derive(Serialize, Deserialize, Debug)]
struct ClientList {
    clientes: Vec<ClientDataJson>,
}

/// Struct for storing formatted client data.
#[derive(Serialize, Deserialize, Debug)]
struct ClientData {
    uuid: String,
    id: String,
    nome: String,
    login: String,
    endereco: String,
    numero: String,
    bairro: String,
    complemento: String,
    cidade: String,
    cep: String,
    estado: String,
    cpf_cnpj: String,
}

/// Struct for form data received from the client selection form.
#[derive(Deserialize, Debug)]
struct ContractForm {
    #[serde(rename = "selected_clients[]")]
    selected_clients: Vec<String>,
    #[serde(rename = "contract_template[]")]
    contract_template: Vec<String>,
}

/// Fetches the JWT token for authentication.
async fn get_jwt_token(
    client_id: &str,
    client_secret: &str,
    url: &str,
) -> Result<String, anyhow::Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!(
        "Basic {}",
        STANDARD.encode(format!("{}:{}", client_id, client_secret))
    );
    debug!("Sending request to get JWT token");

    let res = client
        .get(url)
        .header(AUTHORIZATION, auth_value)
        .send()
        .await?;

    let status = res.status();
    let text = res.text().await?;
    debug!("JWT Response Status: {:?}", status);
    debug!("JWT Response Body: {:?}", text);

    if !status.is_success() {
        return Err(anyhow::anyhow!("Failed to get JWT token: {}", text));
    }

    info!("Successfully obtained JWT token");
    Ok(text) // Directly return the JWT token
}

/// Fetches all clients from the API and formats their data.
async fn fetch_all_clients(jwt_token: String) -> Result<Vec<ClientData>, anyhow::Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    debug!("Fetching all clients");

    let response = client
        .get(LIST_API_URL)
        .header(AUTHORIZATION, format!("Bearer {}", jwt_token))
        .send()
        .await?
        .json::<ClientList>()
        .await?;

    let mut formatted_clients: Vec<ClientData> = Vec::new();

    for mut cliente in response.clientes {
        if cliente.endereco.is_some()
            && cliente.numero.is_some()
            && cliente.bairro.is_some()
            && cliente.complemento.is_some()
            && cliente.cidade.is_some()
            && cliente.cep.is_some()
            && cliente.estado.is_some() {
            match cliente.cpf_cnpj.parse::<Cpf>() {
                Ok(cpf) => {
                    cliente.cpf_cnpj = cpf.to_string();
                    debug!("CPF: {}", cpf.to_string());
                }
                Err(_) => match cliente.cpf_cnpj.parse::<Cnpj>() {
                    Ok(cnpj) => {
                        cliente.cpf_cnpj = cnpj.to_string();
                        debug!("CNPJ: {}", cnpj.to_string());
                    }
                    Err(_) => error!("Invalid CPF/CNPJ: {} for cliente {}", cliente.cpf_cnpj, cliente.login)
                },
            }

            let client = ClientData {
                uuid: cliente.uuid,
                id: cliente.id,
                nome: cliente.nome,
                login: cliente.login,
                endereco: cliente.endereco.unwrap(),
                numero: cliente.numero.unwrap(),
                bairro: cliente.bairro.unwrap(),
                complemento: cliente.complemento.unwrap(),
                cidade: cliente.cidade.unwrap(),
                cep: cliente.cep.unwrap(),
                estado: cliente.estado.unwrap(),
                cpf_cnpj: cliente.cpf_cnpj,
            };

            formatted_clients.push(client);
        }
    }

    Ok(formatted_clients)
}

/// Displays the form for selecting clients and contract templates.
async fn show_form(jwt_token: String) -> Html<String> {
    let clients = fetch_all_clients(jwt_token).await.expect("erro ao buscar clientes do mkauth");
    debug!("show_form: clients: {:?}", clients);
    save_clients_to_json(&clients, CLIENTS_JSON_FILE).expect("erro ao salvar json com os clientes");
    let template = IndexTemplate { clients: &clients };
    let html = template.render().expect("Failed to render template");
    debug!("show_form: template: {:?}", html);
    Html(html)
}

/// Generates contracts based on the selected clients and templates.
async fn generate_contracts(
    Form(form): Form<ContractForm>,
) -> impl axum::response::IntoResponse {
    debug!("generate_contracts: form: {:?}", form);
    let clients = read_clients_from_json(CLIENTS_JSON_FILE).expect("erro ao ler json com os clientes");

    for login in form.selected_clients.iter() {
        let client_data = clients
            .iter()
            .find(|client| &client.login == login)
            .expect("Selected client not found");

        for template in form.contract_template.iter() {
            let data = Local::now().format("%d/%m/%Y").to_string();
            let contract = match template.as_str() {
                "contrato_padrao_fibra" => {
                    let template = ContratoFibraTemplate {
                        client: &client_data,
                        data,
                    };
                    template.render().unwrap()
                }
                "contrato_padrao_voip" => {
                    let template = ContratoVoipTemplate {
                        client: &client_data,
                        data,
                    };
                    template.render().unwrap()
                }
                "contrato_padrao_fibra_voip" => {
                    let template = ContratoFibraVoipTemplate {
                        client: &client_data,
                        data,
                    };
                    template.render().unwrap()
                }
                _ => continue,
            };
            let out_path = format!("contratos/{}_{}.pdf", template, client_data.login);
            convert_html_to_pdf(&contract, &out_path).expect("erro ao converter html para pdf");
        }
    }

    debug!("Contracts generated successfully");
    Redirect::to("/")
}

/// Converts HTML content to a PDF file.
fn convert_html_to_pdf(template: &String, output_path: &str) -> Result<()> {
    debug!("Converting HTML to PDF at path: {}", output_path);
    let mut file = File::create(OUTPUT_HTML_FILE)?;
    file.write_all(template.as_bytes())?;

    let status = Command::new("wkhtmltopdf")
        .arg(OUTPUT_HTML_FILE)
        .arg(&output_path)
        .status()?;

    if status.success() {
        info!("Contract generated and saved to {}", output_path);
    } else {
        error!("Failed to generate PDF {}", output_path);
    }

    Ok(())

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Client Contract Generator</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/tailwindcss/2.2.19/tailwind.min.css">
</head>
<body>
    <div class="container mx-auto mt-5">
        <h1 class="text-3xl font-bold mb-5">Client Contract Generator</h1>
        <form action="/generate-contracts" method="post">
            <div class="mb-4">
                <label for="clients" class="block text-lg font-medium mb-2">Select Clients:</label>
                <table class="min-w-full bg-white border">
                    <thead>
                        <tr>
                            <th class="border px-4 py-2">Select</th>
                            <th class="border px-4 py-2">Login</th>
                            <th class="border px-4 py-2">Name</th>
                        </tr>
                    </thead>
                    <tbody>
                        {% for client in clients %}
                        <tr>
                            <td class="border px-4 py-2"><input type="checkbox" name="selected_clients[]" value="{{ client.login }}" class="form-checkbox h-5 w-5 text-indigo-600"></td>
                            <td class="border px-4 py-2">{{ client.login }}</td>
                            <td class="border px-4 py-2">{{ client.nome }}</td>
                        </tr>
                        {% endfor %}
                    </tbody>
                </table>
            </div>
            <div class="mb-4">
                <label for="templates" class="block text-lg font-medium mb-2">Select Contract Templates:</label>
                <select id="templates" name="contract_template[]" class="form-multiselect block w-full mt-1" multiple>
                    <option value="contrato_padrao_fibra">Contrato Padrão Fibra</option>
                    <option value="contrato_padrao_fibra_voip">Contrato Padrão Fibra+Voip</option>
                    <option value="contrato_padrao_voip">Contrato Padrão Voip</option>
                </select>
            </div>
            <button type="submit" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">Generate Contracts</button>
        </form>
    </div>
</body>
</html>

}

 */