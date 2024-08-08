use sqlx::{query, query_as, PgPool};
use tracing::error;

use crate::financeiro::contrato_model::ClienteContractData;

use super::contrato_model::{ContratoDto, ContratoTemplate, ContratoTemplateDto, ContratoTemplateEditDto};

pub async fn find_all_contrato_templates(pool: &PgPool) -> Result<Vec<ContratoTemplate>, anyhow::Error> {
    match query_as!(ContratoTemplate,"SELECT * FROM contratos_templates")
        .fetch_all(pool)
        .await
    {
        Ok(contratos) => Ok(contratos),
        Err(e) => {
            error!("Failed to fetch contratos: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar contratos"))
        }
    }
}

pub async fn find_contrato_template_by_id(pool: &PgPool, id: i32) -> Result<ContratoTemplate, anyhow::Error> {
    match query_as!(ContratoTemplate,"SELECT * FROM contratos_templates WHERE id = $1",id)
        .fetch_one(pool)
        .await
    {
        Ok(contrato) => Ok(contrato),
        Err(e) => {
            error!("Failed to fetch contrato: {:?}", e);
            Err(anyhow::anyhow!("Erro ao buscar contrato"))
        }
    }
}

pub async fn save_contrato_template(pool: &PgPool, contrato: &ContratoTemplateDto,path:&str) -> Result<(), anyhow::Error> {
    match query!("INSERT INTO contratos_templates (nome,path) VALUES ($1,$2)",contrato.nome,path)
        .execute(pool)
        .await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to insert contrato: {:?}", e);
                Err(anyhow::anyhow!("Erro ao inserir contrato"))
            }
        }
}

pub async fn find_cliente_contract_data(pool: &PgPool, cliente_id: i32) -> Result<ClienteContractData, anyhow::Error> {
    match query_as!(
        ClienteContractData,
        r#"
        SELECT 
            clientes.id, 
            clientes.nome,
            clientes.login, 
            clientes.formatted_cpf_cnpj, 
            clientes.cep, 
            clientes.rua, 
            clientes.numero, 
            clientes.bairro, 
            clientes.cidade, 
            clientes.estado, 
            clientes.complemento, 
            clientes.plano_id,
            contratos_templates.path AS contrato_template_path,
            contratos_templates.nome AS contrato_template_nome,
            contratos_templates.id AS contrato_template_id
        FROM 
            clientes
        JOIN 
            planos ON clientes.plano_id = planos.id
        JOIN 
            contratos_templates ON planos.contrato_template_id = contratos_templates.id
        WHERE 
            clientes.id = $1
        "#, cliente_id)
        .fetch_one(&*pool)
        .await {
            Ok(cliente) => Ok(cliente),

            Err(e) => {
                error!("Failed to fetch cliente contract data: {:?}", e);
                Err(anyhow::anyhow!("Erro ao buscar dados do contrato do cliente"))
            }
        }
}

pub async fn save_contrato(pool:&PgPool,contrato:&ContratoDto) -> Result<(),anyhow::Error> {
    match query!(
        "INSERT INTO contratos (nome, path, template_id, cliente_id) VALUES ($1, $2, $3, $4)",
        contrato.nome,
        contrato.path,
        contrato.template_id,
        contrato.cliente_id
    ).execute(&*pool).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to insert contrato: {:?}", e);
            Err(anyhow::anyhow!("Erro ao inserir contrato"))
        }
    }
}

pub async fn update_contrato_template_in_db(pool:&PgPool,contrato:&ContratoTemplateEditDto,path:&str) -> Result<(),anyhow::Error> {
    match query!(
        "UPDATE contratos_templates SET nome = $1, path = $2 WHERE id = $3",
        contrato.nome,
        path,
        contrato.id
    ).execute(&*pool).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to update contrato template: {:?}", e);
            Err(anyhow::anyhow!("Erro ao atualizar contrato"))
        }
    }
}