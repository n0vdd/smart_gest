use anyhow::Context;
use sqlx::{query, query_as, PgPool};
use tracing::error;

use crate::config::config_model::{EmailConfig, NfConfig};

use super::config_model::{EmailConfigDto, NfConfigDto, Provedor, ProvedorDto};

//There should be only one provedor
//so theres no need to find by id
pub async fn find_provedor(pool: &PgPool) -> Result<Option<Provedor>, anyhow::Error> {
    match query_as!(Provedor, "SELECT * FROM provedor" )
        .fetch_optional(pool)
        .await
    {
        Ok(provedor) => Ok(provedor),
        Err(e) => {
            error!("Failed to fetch provedor: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar provedor"))
        }
    }
}

pub async fn save_provedor_to_db(pool:&PgPool,provedor:&ProvedorDto) -> Result<(),anyhow::Error> {
    match query!("INSERT INTO provedor (nome,cnpj,cep,rua,numero,bairro,cidade,estado,complemento,telefone,email,observacao) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
        provedor.nome,provedor.cnpj,provedor.endereco.cep,provedor.endereco.rua,provedor.endereco.numero,provedor.endereco.bairro,provedor.endereco.cidade
        ,provedor.endereco.estado,provedor.endereco.complemento,provedor.telefone,provedor.email,provedor.observacao)
        .execute(&*pool).await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to insert provedor: {:?}", e);
                Err(anyhow::anyhow!("Erro ao inserir provedor"))
            }
    }
}

pub async fn update_provedor_in_db(pool:&PgPool,provedor:&Provedor) -> Result<(),anyhow::Error>{
    match query!("UPDATE provedor SET nome = $1, cnpj = $2, cep = $3, rua = $4, numero = $5,
        bairro = $6, cidade = $7, estado = $8, complemento = $9, telefone = $10, email = $11, observacao = $12 WHERE id = $13",
        provedor.nome,provedor.cnpj,provedor.cep,provedor.rua,provedor.numero,provedor.bairro,provedor.cidade,provedor.estado,
        provedor.complemento,provedor.telefone,provedor.email,provedor.observacao,provedor.id)
        .execute(&*pool).await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to update provedor: {:?}", e);
                Err(anyhow::anyhow!("Erro ao atualizar provedor"))
            }
        }
}

pub async fn find_nf_config(pool: &PgPool) -> Result<Option<NfConfig>, anyhow::Error> {
    match query_as!(NfConfig, "SELECT * FROM nf_config" )
        .fetch_optional(pool)
        .await
    {
        Ok(nf_config) => Ok(nf_config),
        Err(e) => {
            error!("Failed to fetch nf config: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar nf config"))
        }
    }
}

pub async fn get_nf_config_emails(pool:&PgPool) -> Result<Vec<String>,anyhow::Error> {
    let nf_config = query!("SELECT contabilidade_email FROM nf_config" )
        .fetch_one(pool).await
        .context("Erro ao buscar nf config")?;

    Ok(nf_config.contabilidade_email)
}

pub async fn save_nf_config_to_db(pool:&PgPool,nf_config:&NfConfigDto) -> Result<(),anyhow::Error> {
    match query!("INSERT INTO nf_config (contabilidade_email) VALUES ($1)",
        &nf_config.contabilidade_email)
        .execute(&*pool).await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to insert nf config: {:?}", e);
                Err(anyhow::anyhow!("Erro ao inserir nf config"))
            }
    }
}

pub async fn update_nf_config_in_db(pool:&PgPool,nf_config:&NfConfig) -> Result<(),anyhow::Error> {
    match query!("UPDATE nf_config SET contabilidade_email = $1 WHERE id = $2",
        &nf_config.contabilidade_email, nf_config.id)
        .execute(&*pool).await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to update nf config: {:?}", e);
                Err(anyhow::anyhow!("Erro ao atualizar nf config"))
            }
    }
}

pub async fn find_email_config(pool: &PgPool) -> Result<Option<EmailConfig>, anyhow::Error> {
    match query_as!(EmailConfig, "SELECT * FROM email_config" )
        .fetch_optional(pool)
        .await
    {
        Ok(email_config) => Ok(email_config),
        Err(e) => {
            error!("Failed to fetch email config: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar email config"))
        }
    }
}

pub async fn save_email_config_to_db(pool:&PgPool,email_config:&EmailConfigDto) -> Result<(),anyhow::Error> {
    match query!("INSERT INTO email_config (email,password,host) VALUES ($1,$2,$3)",
        &email_config.email,&email_config.password,&email_config.host)
        .execute(&*pool).await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to insert email config: {:?}", e);
                Err(anyhow::anyhow!("Erro ao inserir email config"))
            }
    }
}

pub async fn update_email_config_in_db(pool:&PgPool,email_config:&EmailConfig) -> Result<(),anyhow::Error> {
    match query!("UPDATE email_config SET email = $1, password = $2, host = $3 WHERE id = $4",
        &email_config.email,&email_config.password,&email_config.host,email_config.id)
        .execute(&*pool).await {
            Ok(_) => Ok(()),

            Err(e) => {
                error!("Failed to update email config: {:?}", e);
                Err(anyhow::anyhow!("Erro ao atualizar email config"))
            }
    }
}

pub async fn get_email_used_in_config(pool:&PgPool) -> Result<String,anyhow::Error> {
    let email = query!("SELECT email FROM email_config")
        .fetch_one(pool).await
        .context("Erro ao buscar email de envio")?.email;

    Ok(email)
}

pub async fn get_nome_used_in_provedor(pool:&PgPool) -> Result<String,anyhow::Error> {
    let nome = query!("SELECT nome FROM provedor")
        .fetch_one(pool).await
        .context("Erro ao buscar nome do provedor")?.nome;

    Ok(nome)
}