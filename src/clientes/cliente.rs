use anyhow::Context;
use chrono::{Datelike, Utc};
use radius::{bloqueia_cliente_radius, delete_cliente_radius};
use sqlx::{query, query_as, PgPool};
use time::{macros::format_description, Duration, Month, PrimitiveDateTime};
use tracing::error;

use crate::financeiro::pagamentos::find_pagamento_confirmado_by_cliente_id_and_date;

use super::cliente_model::{Cliente, ClienteDto, SimpleCliente, TipoPessoa};

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