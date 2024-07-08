//TODO one handler will show a form for uploading the contract html with its name

use std::sync::Arc;

use anyhow::{anyhow, Ok};
use askama::Template;
use axum::{extract::Path, response::{IntoResponse, Redirect}, Extension};
use axum_extra::response::Html;
use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, PgPool};
use tokio::{fs::{DirBuilder, File}, io::AsyncWriteExt, process::Command};
use tracing::error;



//TODO criar modelo de dados usados para gerar o contrato
//terei que buscar como um subset dos dados do cliente(posso pegar pela db)
#[derive(Template)]
#[template(path = "contratos/contrato_padrao_fibra.html")]
pub struct ContratoPadraoFibra {
    client: ClienteContractData,
    data: String
}

#[derive(Template)]
#[template(path = "contratos/contrato_padrao_fibra+voip.html")]
pub struct ContratoPadraoFibraVoip {
    client: ClienteContractData,
    data: String
}

#[derive(Template)]
#[template(path = "contratos/contrato_padrao_voip.html")]
pub struct ContratoPadraoVoip {
    client: ClienteContractData,
    data: String,
}
/// Struct for storing formatted client data.
#[derive(Serialize, Deserialize, Debug,Clone)]
struct ClienteContractData {
    id: i32,
    nome: String,
    login: String,
    //TODO display endereco is used here
    //could do a serde flaten ?
    //or just use the fields
    //will need to deal with option either way
    rua: String,
    numero: Option<String>,
    complemento: Option<String>,
    bairro: String,
    cidade: String,
    estado: String,
    cep: String,
    formatted_cpf_cnpj: String,
    contrato_template_nome: String,
    contrato_template_id: i32,
    plano_id: Option<i32>
}

//Contratos sao exibidos com o nome
//gerados com o template
//e salvam o caminho do arquivo gerado
#[derive(Serialize, Deserialize, Debug,FromRow)]
struct Contrato {
    id: i32,
    nome: String,
    path: String,
    template_id: i32,
    cliente_id: i32,
}

struct ContratoDto {
    nome: String,
    path: String,
    template_id: i32,
    cliente_id: i32,
}
#[derive(Debug,FromRow)]
struct ContratoTemplateDto {
    nome: String,
    path: String
}

#[derive(Serialize,Deserialize,Debug,FromRow)]
pub struct ContratoTemplate {
    pub id: i32,
    pub nome: String,
    pub path: String,
}

//Adiciona os templates usados para gerar os contratos
//Dont need extension and can return a result
pub async fn add_template(pool: &PgPool) -> Result<(),anyhow::Error>{
    let templates = vec![
        ContratoTemplateDto {
            nome: "Contrato Fibra".to_string(),
            path: "contratos/contrato_padrao_fibra.html".to_string()
        },
        ContratoTemplateDto {
            nome: "Contrato Fibra+Voip".to_string(),
            path: "contratos/contrato_padrao_fibra+voip.html".to_string()
        },
        ContratoTemplateDto {
            nome: "Contrato Voip".to_string(),
            path: "contratos/contrato_padrao_voip.html".to_string()
        }];


    for template in templates {
        //Esse erro nao deveria levar o programa a parar
        //deveria ser tratado de forma mais elegante(logado e seguir a execucao do codigo)
        let existente = query_as!(ContratoTemplate,
            "SELECT * FROM contratos_templates WHERE nome = $1",
            template.nome
        ).fetch_optional(&*pool).await.map_err(|e| {
            error!("Failed to fetch template: {:?}", e);
        }).expect("Erro ao buscar template");

        if existente.is_none() {
            query!(
                "INSERT INTO contratos_templates (nome , path) VALUES ($1, $2)",
                template.nome,
                template.path
            )
            .execute(&*pool)
            .await.map_err(|e| {
                error!("Failed to insert template: {:?}", e);
                anyhow!("Failed to insert template")
            })?;
        }
    }
    Ok(())
}

//It should be saved to a temp html and then converted to pdf
//will receive the cliente_id from path
//will need to get the cliente plano to get the template it uses(there is no need for the match)
pub async fn generate_contrato(Extension(pool):Extension<Arc<PgPool>>,Path(cliente_id): Path<i32>) -> impl IntoResponse {
    //fetch cliente data
    let client = query_as!(
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
        .await.map_err(
            |e| {
                error!("Failed to fetch client data: {:?}", e);
                return Html("<p>Failed to fetch client data</p>".to_string());
        }).expect("Erro ao buscar dados do cliente");


    //TODO There should be a better way to do this
    let template = match client.contrato_template_nome.as_str() {
        "Contrato Fibra" => {
            let template = ContratoPadraoFibra { client: client.clone() , data: Local::now().to_string()}.render().map_err(
            |e| {
                error!("Failed to render contract fibra template: {:?}", e);
                e
        }).expect("Erro ao renderizar contrato fibra");
        (template,"".to_string())
        },
        "Contrato Fibra+Voip" => {
            let template = ContratoPadraoFibraVoip { client: client.clone() , data: Local::now().to_string() }.render().map_err(|e| -> _ {
            error!("Failed to render contract fibra+voip template: {:?}", e);
            e
        }).expect("Erro ao renderizar contrato fibra+voip");
            let template2 = ContratoPadraoVoip { client: client.clone() , data: Local::now().to_string() }
            .render().map_err(|e| -> _ {
                error!("Failed to render contract voip template: {:?}", e);
                e}).expect("Erro ao renderizar contrato voip");
            (template,template2)
        },
        "Contrato Voip" => {  let template = ContratoPadraoFibraVoip { client: client.clone() , data: Local::now().to_string() }.render().map_err(|e| -> _ {
            error!("Failed to render contract fibra+voip template: {:?}", e);
            e
        }).expect("Erro ao renderizar contrato fibra+voip");
            let template2 = ContratoPadraoVoip { client: client.clone() , data: Local::now().to_string() }
            .render().map_err(|e| -> _ {
                error!("Failed to render contract voip template: {:?}", e);
                e}).expect("Erro ao renderizar contrato voip");
            (template,template2)
        },
        _ => return Html("<p>Invalid contract template</p>").into_response()
    };


    let dir_path = format!("contratos/{}", client.nome);
    DirBuilder::new().recursive(true).create(&dir_path).await.map_err(|e| {
        error!("Failed to create directory: {:?}", e);
        e
    }).expect("Erro ao criar diretorio");

    // Save the rendered HTML to a temporary file
    let html_file_path = format!("/tmp/contract_{}.html", cliente_id);


    //TODO there should be a better way to do this
    if template.0 != "" {
        File::create(&html_file_path).await.map_err(|e| {
            error!("Failed to create HTML file: {:?}", e);
            e
        }).expect("Erro ao criar arquivo html")
        .write_all(template.0.as_bytes()).await.map_err(|e| {
            error!("Failed to write HTML to file: {:?}", e);
            e
        }).expect("Erro ao salvar html em arquivo");
    } else if template.1 != "" {
        File::create(&html_file_path).await.map_err(|e| {
            error!("Failed to create HTML file: {:?}", e);
            e
        }).expect("Erro ao criar arquivo html")
        .write_all(template.0.as_bytes()).await.map_err(|e| {
            error!("Failed to write HTML to file: {:?}", e);
            e
        }).expect("Erro ao salvar html em arquivo");
    }  

    // Save the contract to the filesystem
    let pdf_file_path = format!("contratos/{}/{}-{}-{}.pdf", client.nome, client.contrato_template_nome, Local::now().to_string(),client.login);
    // Convert the HTML to PDF using the wkhtmltopdf command
    Command::new("wkhtmltopdf")
        .arg(&html_file_path)
        .arg(&pdf_file_path)
        .output()
        .await
        .map_err(|e| {
            error!("Failed to convert HTML to PDF: {:?}", e);
            e
    }).expect("Erro ao converter html para pdf");

    // Save the contract path to the database
    let contrato = ContratoDto {
        nome: client.nome,
        path: pdf_file_path,
        template_id: client.contrato_template_id,
        cliente_id: client.id,
    };

    query!(
        "INSERT INTO contratos (nome, path, template_id, cliente_id) VALUES ($1, $2, $3, $4)",
        contrato.nome,
        contrato.path,
        contrato.template_id,
        contrato.cliente_id
    ).execute(&*pool).await.map_err(|e| {
        error!("Failed to save contract path to database: {:?}", e);
        e
    }).expect("Erro ao salvar caminho do contrato no banco de dados");

    Redirect::to("/cliente").into_response()
}
