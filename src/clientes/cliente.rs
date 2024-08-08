use std::str::FromStr;

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Datelike, Utc};
use cnpj::Cnpj;
use cpf::Cpf;
use radius::{bloqueia_cliente_radius, delete_cliente_radius};
use reqwest::header::AUTHORIZATION;
use sqlx::{query, query_as, PgPool};
use time::{macros::format_description, Duration, Month, PrimitiveDateTime};
use tokio::{fs::{File, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::{debug, error, info};


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



use crate::{clientes::cliente_model::ClientList, config::endereco_model::EnderecoDto, financeiro::pagamentos::find_pagamento_confirmado_by_cliente_id_and_date};

use super::cliente_model::{ClientDataJson, Cliente, ClienteDto, SimpleCliente, TipoPessoa};

//TODO add pagination and this kind of thing, will need better htmx on list
pub async fn find_all_clientes(pool:&PgPool) -> Result<Vec<Cliente>,anyhow::Error> {
    match query_as!(
        Cliente,
        "SELECT * FROM clientes"
    )
    .fetch_all(pool)
    .await {
        Ok(clientes) => Ok(clientes),
        Err(e) => {
            error!("Failed to fetch clientes: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar clientes"))
        }
    }
}

//TODO find where this is used, give it a better name
pub async fn get_all_clientes(pool: &PgPool) -> Result<Vec<SimpleCliente>, anyhow::Error> {
    match query_as!(SimpleCliente, "SELECT id,login FROM clientes")
        .fetch_all(pool)
        .await {
            Ok(clientes) => Ok(clientes),

            Err(e) => {
                error!("Failed to fetch clientes: {:?}", e);
                Err(anyhow::anyhow!("Erro ao buscar clientes"))
            }
        }
}

pub async fn get_cliente_login_by_id(pool: &PgPool, id: i32) -> Result<String, anyhow::Error> {
    match query_as!(SimpleCliente, "SELECT id,login FROM clientes WHERE id = $1", id)
        .fetch_one(pool)
        .await {
            Ok(cliente) => Ok(cliente.login),

            Err(e) => {
                error!("Failed to fetch clientes: {:?}", e);
                Err(anyhow::anyhow!("Erro ao buscar clientes"))
            }
        }
}

pub async fn delete_cliente_by_id(pool: &PgPool, id: i32) -> Result<(), anyhow::Error> {
    let login = get_cliente_login_by_id(pool, id).await?;

    delete_cliente_radius(login).await?;

    query!("DELETE FROM clientes WHERE id = $1", id)
        .execute(pool)
        .await.context("Erro ao deletar cliente")?;

    Ok(())
}

pub async fn find_cliente_by_id(pool: &PgPool, id: i32) -> Result<Cliente, anyhow::Error> {
    match query_as!(Cliente, "SELECT * FROM clientes WHERE id = $1", id)
        .fetch_one(pool)
        .await {
            Ok(cliente) => Ok(cliente),

            Err(e) => {
                error!("Failed to fetch cliente: {:?}", e);
                Err(anyhow::anyhow!("Erro ao buscar cliente"))
            }
        }
}

pub async fn update_cliente_by_id(cliente: &Cliente, pool: &PgPool) -> Result<(),anyhow::Error> {
    match query!(
        "UPDATE clientes SET tipo = $1, nome = $2, email = $3, cpf_cnpj = $4, formatted_cpf_cnpj = $5,
        telefone = $6, login = $7, senha = $8, cep = $9, rua = $10, numero = $11, bairro = $12,
        complemento = $13, cidade = $14, estado = $15, ibge_code = $16, mikrotik_id = $17,
        plano_id = $18, gera_nf = $19, gera_dici = $20, add_to_asaas = $21 WHERE id = $22",
        cliente.tipo,
        cliente.nome,
        cliente.email,
        cliente.cpf_cnpj,
        cliente.formatted_cpf_cnpj,
        cliente.telefone,
        cliente.login,
        cliente.senha,
        cliente.cep,
        cliente.rua,
        cliente.numero,
        cliente.bairro,
        cliente.complemento,
        cliente.cidade,
        cliente.estado,
        cliente.ibge_code,
        cliente.mikrotik_id,
        cliente.plano_id,
        cliente.gera_nf,
        cliente.gera_dici,
        cliente.add_to_asaas,
        cliente.id
    ).execute(pool).await {
        Ok(_) => Ok(()),

        Err(e) => {
            error!("Failed to update cliente: {:?}", e);
            Err(anyhow::anyhow!("Erro ao atualizar cliente"))
        }
    }
}

pub async fn save_cliente(cliente: &ClienteDto, pool: &PgPool) -> Result<(), anyhow::Error> {
    match query!(
        "INSERT INTO clientes (
            tipo, nome, email, cpf_cnpj, formatted_cpf_cnpj, telefone, login, senha, 
            mikrotik_id, plano_id, cep, rua, numero, bairro, complemento, cidade, estado, ibge_code,gera_dici,gera_nf,add_to_asaas
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,$19,$20,$21
        )",
        cliente.tipo.as_bool(),
        cliente.nome,
        cliente.email,
        //cpf_cnpj nao formatado sera usado para nota_fiscal
        cliente.cpf_cnpj,
        //cpf_cnpj formatado sera usado para exibir na pagina e para o contrato
        cliente.formatted_cpf_cnpj,
        cliente.telefone,
        //login e senha usado para controle de acesso pelo radius
        cliente.login,
        cliente.senha,
        cliente.mikrotik_id,
        cliente.plano_id,
        cliente.endereco.cep,
        cliente.endereco.rua,
        cliente.endereco.numero,
        cliente.endereco.bairro,
        cliente.endereco.complemento,
        cliente.endereco.cidade,
        cliente.endereco.estado,
        cliente.endereco.ibge,
        cliente.gera_dici,
        cliente.gera_nf,
        cliente.add_to_asaas
    ).execute(pool).await {
        Ok(_) => Ok(()),

        Err(e) => {
            error!("Failed to save cliente: {:?}", e);
            Err(anyhow::anyhow!("Erro ao salvar cliente"))
        }
    }
}

//used for dici generation
//param: date, a date to be compared on the db with the time the cliente was created
//Selects the tipo of the cliente(Pessoa Fisica ou Juridica) for all the clientes created before the given date
//Returns a List with all the tipos of the clientes created before the given date
pub async fn fetch_tipo_clientes_before_date_for_dici(
    pool: &PgPool,
    date: PrimitiveDateTime
) -> Result<Vec<TipoPessoa>,anyhow::Error> {
    //Get the tipo of the cliente created before a given data
    let tipos = query!("SELECT tipo FROM clientes WHERE created_at < $1 and gera_dici = false", date)
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

//TODO entender melhor como sao as datas de cobranca do asaas
//?a cobranca do cartao confirma no dia do pagamento?
//?a cobranca do boleto a pessoa consegue pagar a partir do momento que ele e gerada?quando ela e gerada?
pub async fn bloqueia_clientes_atrasados(pool: &PgPool) -> Result<(),anyhow::Error>{
    let clientes = get_all_clientes(&*pool).await.map_err(|e| {
        error!("Failed to fetch clientes: {:?}", e);
        anyhow::anyhow!("Failed to fetch all clientes")
    }).expect("Erro ao buscar clientes");

    for cliente in clientes {
        //Format chrono to primitiveDateTive
        let format = format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");

        let prev_date = PrimitiveDateTime::parse(chrono::Utc::now().to_string().as_str(), format)?; 
        let prev_month = Utc::now().month() - 1;
        //Voltei um mes, ainda no dia 12
        let prev_date = prev_date.replace_month(Month::try_from(prev_month as u8)?)?;
        //fui para o dia 25
        //dia 25 mes passado
        let prev_date = prev_date.checked_add(Duration::days(13)).unwrap();

        //dia 12 do mes atual
        let date = PrimitiveDateTime::parse(chrono::Utc::now().to_string().as_str(), format)?;
        //TODO conferir a partir de quando o cliente conseguiria pagar o boleto/cartao para deixar uma data mais exata
        //caso o cliente nao tenha um pagamento confirmado do dia 25 do mes passado ate hoje
        //bloqueia o cliente no radius
        let payment = find_pagamento_confirmado_by_cliente_id_and_date(&pool, cliente.id, prev_date, date).await.map_err(|e| {
            error!("Failed to fetch pagamentos: {:?}", e);
            anyhow::anyhow!("Failed to fetch pagamentos")
        }).expect("Erro ao buscar pagamentos");

        if payment.is_none() {
            bloqueia_cliente_radius(&cliente.login).await.map_err(|e| {
                error!("Failed to block cliente: {:?}", e);
                anyhow::anyhow!("Failed to block cliente")
            }).expect("Erro ao bloquear cliente");
        }
    }

    Ok(())
}

pub async fn find_cliente_in_mikrotik(pool: &PgPool, mikrotik_id: i32) -> Result<Vec<Cliente>, anyhow::Error> {
    match query_as!(Cliente, "SELECT * FROM clientes WHERE mikrotik_id = $1", mikrotik_id)
        .fetch_all(pool)
        .await {
            Ok(cliente) => Ok(cliente),

            Err(e) => {
                error!("Failed to fetch cliente: {:?}", e);
                Err(anyhow::anyhow!("Erro ao buscar cliente"))
            }
        }
}

pub async fn find_cliente_by_cpf_cnpj(pool: &PgPool, cpf_cnpj: &str) -> Result<Cliente, anyhow::Error> {
    match query_as!(Cliente, "SELECT * FROM clientes WHERE cpf_cnpj = $1", cpf_cnpj)
        .fetch_one(pool)
        .await {
            Ok(cliente) => Ok(cliente),

            Err(e) => {
                error!("Failed to fetch cliente: {:?}", e);
                Err(anyhow::anyhow!("Erro ao buscar cliente"))
            }
        }
}

//TODO import clients from mikrotik
/// Fetches the JWT token for authentication.
async fn get_jwt_token() -> Result<String, anyhow::Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!(
        "Basic {}",
        STANDARD.encode(format!("{}:{}", CLIENT_ID, CLIENT_SECRET))
    );

    debug!("Sending request to get JWT token");

    let res = client
        .get(AUTH_URL)
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
async fn fetch_all_clients(jwt_token: String) -> Result<ClientList, anyhow::Error> {
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

    Ok(response)
}

/// Saves the list of clients to a JSON file.
async fn save_clients_to_json(clients: &ClientList, file_path: &str) -> Result<(),anyhow::Error> {
    debug!("Saving clients to JSON file at path: {}", file_path);

    let data = serde_json::to_vec_pretty(clients)
        .context("Failed to serialize clients to JSON")?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .await
        .context("Failed to open file for writing")?;

    file.write_all(&data)
        .await
        .context("Failed to write data to file")?;

    debug!("Saved clients to JSON file");
    Ok(())
}
/// Reads the list of clients from a JSON file.
pub async fn read_clients_from_json(file_path: &str) -> Result<ClientList,anyhow::Error> {
    debug!("Reading clients from JSON file at path: {}", file_path);

    let mut file = File::open(file_path)
        .await
        .context("Failed to open file for reading")?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .await
        .context("Failed to read data from file")?;

    let clients = serde_json::from_slice(&data)
        .context("Failed to deserialize clients from JSON")?;

    Ok(clients)
}

pub async fn import_mikauth_clientes(pool:&PgPool) -> Result<(),anyhow::Error>{
    let clients = read_clients_from_json("clientes.json").await?;

    for cliente in clients.clientes {
        let tipo: bool;
        let cpf_cnpj = match Cpf::from_str(&cliente.cpf_cnpj) {
            Ok(cpf) => {
                tipo = TipoPessoa::as_bool(&TipoPessoa::PessoaFisica);
                cpf.to_string()},
            Err(_) => {
                tipo = TipoPessoa::as_bool(&TipoPessoa::PessoaJuridica);
                Cnpj::from_str(&cliente.cpf_cnpj)?.to_string()
            },
        };

        let endereco = EnderecoDto {
            cep: cliente.cep.unwrap_or("00000000".to_string()), 
            rua: cliente.endereco.unwrap_or("Rua".to_string()),
            numero: cliente.numero,
            bairro: cliente.bairro.unwrap_or("Bairro".to_string()),
            cidade: cliente.cidade.unwrap_or("Cidade".to_string()),
            estado: cliente.estado.unwrap_or("Estado".to_string()),
            complemento: cliente.complemento,
            ibge: "0000000".to_string(),
        };

        let new_cliente = ClienteDto {
            tipo: TipoPessoa::from_bool(tipo),
            nome: cliente.nome,
            email: cliente.email.unwrap_or("amorimmaori@gmail.com".to_string()),
            telefone: "00000000000".to_string(),
            cpf_cnpj: cpf_cnpj,
            formatted_cpf_cnpj: cliente.cpf_cnpj,
            login: cliente.login,
            senha: cliente.senha,
            mikrotik_id: 1,
            plano_id: 3,
            contrato_id: None,
            endereco: endereco,
            gera_nf: false,
            gera_dici: false,
            add_to_asaas: false,
        };

        save_cliente(&new_cliente, &pool).await?;
    }
    Ok(())
}