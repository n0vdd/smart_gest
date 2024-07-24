//TODO one handler will show a form for uploading the contract html with its name

use std::sync::Arc;

use anyhow::{anyhow, Ok};
use askama::Template;
use axum::{extract::Path, response::{IntoResponse, Redirect}, Extension};
use axum_extra::response::Html;
use chrono::{Datelike, Local};
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

//Adiciona as templates usadas para gerar os contratos ao banco de dados
//Apenas caso as mesmas ainda nao existam no banco
//Sao valores hard_coded
//TODO talvez possa fazer isso pelo sistema, mas acho que ficaria mais trabalhoso no momento
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


    //Passa por todas as templates hardcoded, checando se a mesma existe pelo nome
    //caso nao ache nada,insere a mesma no banco
    for template in templates {
        //Checa se a template ja existe
        let existente = query_as!(ContratoTemplate,
            "SELECT * FROM contratos_templates WHERE nome = $1",
            template.nome
        ).fetch_optional(&*pool).await.map_err(|e| {
            error!("Failed to fetch template: {:?}", e);
            anyhow!("Erro ao buscar template {:?} no banco de dados",template)
        })?;

        //Insere a template no bd caso ela nao exista
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

//Recebe a id do cliente que ira ser responsavel pelo cliente(pelo button gerar contrato)
//acha os dados necessarios para gerar o contrato do cliente usando sua id(endereco,cpf_cnpj e plano(consege a template do contrato por ele))
//Gera uma String com a data do dia para adicionar ao contrato tambem
//Gera os templates do contrato baseado no nome do mesmo
//Cria um diretorio com o nome do cliente para salvar seus contratos
//Cria um arquivo temporario para gerar/salvar a(s) template(s) do contrato
//Converte esse html temporario em um pdf na pasta do cliente 
//adiciona nome/caminho e cliente responsavel pelo cliente ao banco de dados
//Retorna um redirect do usuario para a listagem de clientes
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


    //Formata a data que sera colocada no contrato
    let day = Local::now().day();
    let month = Local::now().month();
    let year = Local::now().year();
    let data = format!("{}/{}/{}",day,month,year);

    //TODO There should be a better way to do this
    //Compara o nome da template para saber qual contrato sera gerado
    //Tenho que retornar uma tuple com 2 contratos devido a template fibra+voip(usa 2 contratos)
    let template = match client.contrato_template_nome.as_str() {
        "Contrato Fibra" => {
            let template = ContratoPadraoFibra { client: client.clone() , data}.render().map_err(
            |e| {
                error!("Failed to render contract fibra template: {:?}", e);
                anyhow!("Falha ao renderizar template de contrato para fibra")
        }).expect("Erro ao renderizar contrato fibra");
        (template,"".to_string())
        },
        "Contrato Fibra+Voip" => {
            let template = ContratoPadraoFibraVoip { client: client.clone() , data:data.clone()}.render().map_err(|e| -> _ {
            error!("Failed to render contract fibra+voip template: {:?}", e);
            anyhow!("Falha ao renderizar template do contrato de fibra+voip")
        }).expect("Erro ao renderizar contrato fibra+voip");
            let template2 = ContratoPadraoVoip { client: client.clone() , data}
            .render().map_err(|e| -> _ {
                error!("Failed to render contract voip template: {:?}", e);
                anyhow!("Falha ao renderizar template de contrato do voip")
            }).expect("Erro ao renderizar contrato voip");
            (template,template2)
        },
        "Contrato Voip" => {  
            let template = ContratoPadraoVoip { client: client.clone() , data}
            .render().map_err(|e| -> _ {
                error!("Failed to render contract voip template: {:?}", e);
                anyhow!("Falha ao renderizar template do contrato de voip")
            }).expect("Erro ao renderizar contrato voip");
            (template,"".to_string())
        },
        _ => return Html("<p>Invalid contract template</p>").into_response()
    };


    //Cria o diretorio com o nome do cliente para salvar os contratos relacionados ao mesmo
    let dir_path = format!("contratos/{}", client.nome);
    DirBuilder::new().recursive(true).create(&dir_path).await.map_err(|e| {
        error!("Failed to create directory: {:?}", e);
        anyhow!("Falha ao criar diretorio para salvar contratos do cliente {e}")
    }).expect("Erro ao criar diretorio");

    // Save the rendered HTML to a temporary file
    let html_file_path = format!("/tmp/contract_{}.html", cliente_id);


    //TODO there should be a better way to do this
    //Checa se a template tem um contrato antes de salvar o mesmo para um arquivo
    if template.0 != "" {
        File::create(&html_file_path).await.map_err(|e| {
            error!("Failed to create HTML file: {:?}", e);
            anyhow!("Falha ao criar o arquivo html {html_file_path} para o contrato")
        }).expect("Erro ao criar arquivo html")
        .write_all(template.0.as_bytes()).await.map_err(|e| {
            error!("Failed to write HTML to file: {:?}", e);
            anyhow!("Falha ao escrever dados do template html para o arquivo temporario")
        }).expect("Erro ao salvar html em arquivo");
    } else if template.1 != "" {
        File::create(&html_file_path).await.map_err(|e| {
            error!("Failed to create HTML file: {:?}", e);
            anyhow!("Falha ao criar o arquivo html {html_file_path} para o contrato")
        }).expect("Erro ao criar arquivo html")
        .write_all(template.1.as_bytes()).await.map_err(|e| {
            error!("Failed to write HTML to file: {:?}", e);
            anyhow!("Falha ao escrever dados do template html para o arquivo temporario")
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
            anyhow!("Falha ao converter html temporario: {html_file_path} para o arquivo pdf:{pdf_file_path}")
    }).expect("Erro ao converter html para pdf");

    // Save the contract path to the database
    let contrato = ContratoDto {
        nome: client.nome,
        path: pdf_file_path,
        template_id: client.contrato_template_id,
        cliente_id: client.id,
    };

    //Salva o nome,caminho e cliente responsavel pelo contrato para a base de dados
    query!(
        "INSERT INTO contratos (nome, path, template_id, cliente_id) VALUES ($1, $2, $3, $4)",
        contrato.nome,
        contrato.path,
        contrato.template_id,
        contrato.cliente_id
    ).execute(&*pool).await.map_err(|e| {
        error!("Failed to save contract path to database: {:?}", e);
        anyhow!("Falha ao salvar os dados do contrato para o banco de dados")
    }).expect("Erro ao salvar caminho do contrato no banco de dados");

    Redirect::to("/cliente").into_response()
}
