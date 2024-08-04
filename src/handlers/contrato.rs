use std::{borrow::Borrow, sync::Arc};

use axum::{extract::Path, response::{IntoResponse, Redirect}, Extension};
use axum_extra::{extract::Form, response::Html};
use chrono::{Datelike, Local};
use sqlx::{query, query_as, PgPool};
use tera::{Context, Tera};
use tokio::{fs::{DirBuilder, File}, io::AsyncWriteExt, process::Command};
use tracing::error;

use crate::{models::contrato::{ClienteContractData, ContratoDto, ContratoTemplate, ContratoTemplateDto}, TEMPLATES};

pub async fn show_contrato_template_list(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    let templates = query_as!(ContratoTemplate, "SELECT * FROM contratos_templates")
        .fetch_all(&*pool)
        .await.map_err(|e| {
            error!("Failed to fetch contrato templates: {:?}", e);
            anyhow::anyhow!("Failed to fetch contrato templates")
        }).expect("Failed to fetch contrato templates");

    let mut context = Context::new();
    context.insert("templates", &templates);

    let template = TEMPLATES .render("contrato_template_list.html", &context)
        .map_err(|e| {
            error!("Failed to render contrato template list: {:?}", e);
            anyhow::anyhow!("Failed to render contrato template list")
        }).expect("Failed to render contrato template list");

    Html(template)
}

//in the bottom of the form it should show what data can be used on the template like 
//{{ date }} {{ cliente.nome }} etc,etc
pub async fn show_contrato_template_add_form() -> impl IntoResponse {
    let template = TEMPLATES.render("contrato_template_add_form.html", &Context::new())
        .map_err(|e| {
            error!("Failed to render contrato template add form: {:?}", e);
            anyhow::anyhow!("Failed to render contrato template add form")
        }).expect("Failed to render contrato template add form");

    Html(template)
}

pub async fn show_contrato_template_edit_form(Extension(pool):Extension<Arc<PgPool>>,Path(id):Path<i32>) -> impl IntoResponse {
    let template = query_as!(ContratoTemplate,"SELECT * FROM contratos_templates WHERE id = $1", id)
        .fetch_one(&*pool)
        .await.map_err(|e| {
            error!("Failed to fetch contrato template: {:?}", e);
            anyhow::anyhow!("Failed to fetch contrato template")
        }).expect("Failed to fetch contrato template");

    let data = tokio::fs::read(&template.path).await.map_err(|e| {
        error!("Failed to read template file: {:?}", e);
        anyhow::anyhow!("Failed to read template file")
    }).expect("Failed to read template file");
    let data = String::from_utf8(data).map_err(|e| {
        error!("Failed to convert template data to string: {:?}", e);
        anyhow::anyhow!("Failed to convert template data to string")
    }).expect("Failed to convert template data to string");

    let mut context = Context::new();
    //This should be a string i think
    context.insert("data", &data);
    context.insert("template", &template);


    let template = TEMPLATES.render("contrato_template_edit.html",&context)
        .map_err(|e| {
            error!("Failed to render contrato template edit form: {:?}", e);
            anyhow::anyhow!("Failed to render contrato template edit form")
        }).expect("Failed to render contrato template edit form");

    Html(template)
}

//Save the path and name of the template used to generate the contract to the db
//save the template itself to path
pub async fn add_contrato_template(Extension(pool):Extension<Arc<PgPool>>,
    Form(template): Form<ContratoTemplateDto>) -> impl IntoResponse {
    //Save the template to the filesystem
    let path = format!("templates/contratos/{}.html", template.nome);

    File::create(&path).await.map_err(|e| {
        error!("Failed to create template file: {:?}", e);
        anyhow::anyhow!("Failed to create template file")
    }).expect("Failed to create template file")
    .write_all(template.data.as_bytes()).await.map_err(|e| {
        error!("Failed to write template data to file: {:?}", e);
        anyhow::anyhow!("Failed to write template data to file")
    }).expect("Failed to write template data to file");

    query!(
        "INSERT INTO contratos_templates (nome, path) VALUES ($1, $2)",
        template.nome,
        path
    ).execute(&*pool).await.map_err(|e| {
        error!("Failed to save template to database: {:?}", e);
        anyhow::anyhow!("Failed to save template to database")
    }).expect("Failed to save template to database");

    //After adding a new template we need to reload the templates
    //So that we can latter render it
    let mut templates = TEMPLATES.clone();
    templates.full_reload().map_err(|e| {
        error!("Failed to reload templates: {:?}", e);
        anyhow::anyhow!("Failed to reload templates")
    }).expect("Failed to reload templates");

    Redirect::to("/financeiro/contrato_template")
}

//Adiciona as templates usadas para gerar os contratos ao banco de dados
//Apenas caso as mesmas ainda nao existam no banco
//Sao valores hard_coded
//TODO talvez possa fazer isso pelo sistema, mas acho que ficaria mais trabalhoso no momento
//TODO criar templates pelo sistema vai envolver criar templates do askama e coisas do tipo
//nao sei o quanto seria viavel, mas seria necessario, talvez consigar criar trais e afins, sla
/* 
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
*/
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
pub async fn generate_contrato(Extension(pool):Extension<Arc<PgPool>>,Path(cliente_id): Path<i32>,) -> impl IntoResponse {
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
    let tera= Tera::new("/templates/contratos/*").map_err(|e| {
        error!("Failed to compile contrato templates: {:?}", e);
        anyhow::anyhow!("Failed to compile contrato templates")
    }).expect("Failed to compile contrato templates");
    let mut context = Context::new();
    context.insert("cliente", &client);
    context.insert("date", &data);

    let template = tera.render(&client.contrato_template_path, &context).map_err(|e| {
        error!("Failed to render contrato template: {:?}", e);
        anyhow::anyhow!("Failed to render contrato template")
    }).expect("Failed to render contrato template");

    //Cria o diretorio com o nome do cliente para salvar os contratos relacionados ao mesmo
    let dir_path = format!("contratos/{}", client.nome);
    DirBuilder::new().recursive(true).create(&dir_path).await.map_err(|e| {
        error!("Failed to create directory: {:?}", e);
        anyhow::anyhow!("Falha ao criar diretorio para salvar contratos do cliente {e}")
    }).expect("Erro ao criar diretorio");

    // Save the rendered HTML to a temporary file
    let html_file_path = format!("/tmp/contract_{}.html", cliente_id);

    tokio::fs::write(&html_file_path, template).await.map_err(|e| {
        error!("Failed to write temporary HTML file: {:?}", e);
        anyhow::anyhow!("Falha ao escrever html temporario: {html_file_path}")
    }).expect("Erro ao escrever html temporario");

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
            anyhow::anyhow!("Falha ao converter html temporario: {html_file_path} para o arquivo pdf:{pdf_file_path}")
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
        anyhow::anyhow!("Falha ao salvar os dados do contrato para o banco de dados")
    }).expect("Erro ao salvar caminho do contrato no banco de dados");

    Redirect::to("/cliente").into_response()
}
