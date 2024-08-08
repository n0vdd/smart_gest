use std::sync::Arc;

use axum::{extract::Path, response::{IntoResponse, Redirect}, Extension};
use axum_extra::{extract::Form, response::Html};
use chrono::{Datelike, Local};
use sqlx::PgPool;
use tera::Context;
use tokio::{fs::{DirBuilder, File}, io::AsyncWriteExt, process::Command};
use tracing::error;

use crate::TEMPLATES;

use super::{contrato::{find_all_contrato_templates, find_cliente_contract_data, find_contrato_template_by_id, save_contrato, save_contrato_template, update_contrato_template_in_db}, contrato_model::{ContratoDto, ContratoTemplate, ContratoTemplateDto, ContratoTemplateEditDto}};

pub async fn show_contrato_template_list(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    let templates = find_all_contrato_templates(&pool).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar templates na db");

    let mut context = Context::new();
    context.insert("templates", &templates);

    let template = TEMPLATES.lock().await;
    match template.render("contrato_template/contrato_template_list.html", &context) {
        Ok(template) => Html(template).into_response(),
        Err(e) => {
            error!("Failed to render contrato template list template: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//in the bottom of the form it should show what data can be used on the template like 
//{{ date }} {{ cliente.nome }} etc,etc
pub async fn show_contrato_template_add_form() -> impl IntoResponse {
    let template = TEMPLATES.lock().await;
    match template.render("contrato_template/contrato_template_add_form.html", &Context::new()) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render contrato template add form: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn show_contrato_template_edit_form(Extension(pool):Extension<Arc<PgPool>>,Path(id):Path<i32>) -> impl IntoResponse {
    let template = find_contrato_template_by_id(&pool, id).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar contrato template na db"); 

    let data = tokio::fs::read(&template.path).await.map_err(|e| {
        error!("Failed to read template file: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to read template file".to_string())
    }).expect("Failed to read template file");

    let data = String::from_utf8(data).map_err(|e| {
        error!("Failed to convert template data to string: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to convert template data to string".to_string())
    }).expect("Failed to convert template data to string");

    let mut context = Context::new();
    //This should be a string i think
    context.insert("data", &data);
    context.insert("template", &template);

    let template = TEMPLATES.lock().await;
    match template.render("contrato_template/contrato_template_edit.html",&context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render contrato template edit form: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//Save the path and name of the template used to generate the contract to the db
//save the template itself to path
pub async fn add_contrato_template(Extension(pool):Extension<Arc<PgPool>>,
    Form(contrato): Form<ContratoTemplateDto>) -> impl IntoResponse {
    //Save the template to the filesystem
    let path = format!("templates/contratos/{}.html", contrato.nome);

    File::create(&path).await.map_err(|e| {
        error!("Failed to create template file: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to create template file".to_string())
    }).expect("Failed to create template file")
    .write_all(contrato.data.as_bytes()).await.map_err(|e| {
        error!("Failed to write template data to file: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to write template data to file".to_string())
    }).expect("Failed to write template data to file");

    save_contrato_template(&pool, &contrato,&path).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao salvar template de contrato para o banco de dados");

    //After adding a new template we need to reload the templates
    //So that we can latter render it
    let mut templates = TEMPLATES.lock().await;

    templates.full_reload().map_err(|e| {
        error!("Failed to reload templates: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to reload templates")
    }).expect("Failed to reload templates");

    Redirect::to("/financeiro/contrato_template")
}

pub async fn update_contrato_template(Extension(pool): Extension<Arc<PgPool>>,
    Form(contrato): Form<ContratoTemplateEditDto>) -> impl IntoResponse {

    let path = format!("templates/contratos/{}.html", contrato.nome);

    update_contrato_template_in_db(&pool, &contrato,&path).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao atualizar template de contrato no banco de dados");

    File::create(&path).await.map_err(|e| {
        error!("Failed to create template file: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to create template file".to_string())
    }).expect("Failed to create template file")
    .write_all(contrato.data.as_bytes()).await.map_err(|e| {
        error!("Failed to write template data to file: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to write template data to file".to_string())
    }).expect("Failed to write template data to file");

    //After adding a new template we need to reload the templates
    //So that we can latter render it
    let mut templates = TEMPLATES.lock().await;
    templates.full_reload().expect("Failed to reload templates");

    Redirect::to("/financeiro/contrato_template")
}
//Adiciona as templates usadas para gerar os contratos ao banco de dados
//Apenas caso as mesmas ainda nao existam no banco
//Sao valores hard_coded
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
    let cliente = find_cliente_contract_data(&pool, cliente_id).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar dados do contrato do cliente");

    //Formata a data que sera colocada no contrato
    let day = Local::now().day();
    let month = Local::now().month();
    let year = Local::now().year();
    let data = format!("{}/{}/{}",day,month,year);

    let mut context = Context::new();
    context.insert("cliente", &cliente);
    context.insert("date", &data);

    //TODO deal with error on html
    let template= TEMPLATES.lock().await;
    let template = template.render(&cliente.contrato_template_path, &context).map_err(|e| {
        error!("Failed to render contrato template: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to render contrato template")
    }).expect("Failed to render contrato template");

    //Cria o diretorio com o nome do cliente para salvar os contratos relacionados ao mesmo
    let dir_path = format!("contratos/{}", cliente.nome);

    DirBuilder::new().recursive(true).create(&dir_path).await.map_err(|e| {
        error!("Failed to create directory: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to create contratos directory")
    }).expect("Erro ao criar diretorio");

    // Save the rendered HTML to a temporary file
    let html_file_path = format!("/tmp/contract_{}.html", cliente_id);

    tokio::fs::write(&html_file_path, template).await.map_err(|e| {
        error!("Failed to write temporary HTML file: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to write temporary HTML file");
    }).expect("Erro ao escrever html temporario");

    // Save the contract to the filesystem
    let pdf_file_path = format!("contratos/{}/{}-{}-{}.pdf", cliente.nome, cliente.contrato_template_nome, Local::now().to_string(),cliente.login);

    // Convert the HTML to PDF using the wkhtmltopdf command
    Command::new("wkhtmltopdf")
        .arg(&html_file_path)
        .arg(&pdf_file_path)
        .output()
        .await
        .map_err(|e| {
            error!("Failed to convert HTML to PDF: {:?}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,"Failed to convert HTML to PDF");
    }).expect("Erro ao converter html para pdf");

    // Save the contract path to the database
    let contrato = ContratoDto {
        nome: cliente.nome,
        path: pdf_file_path,
        template_id: cliente.contrato_template_id,
        cliente_id: cliente.id,
    };

    //Salva o nome,caminho e cliente responsavel pelo contrato para a base de dados
    save_contrato(&pool, &contrato).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao salvar contrato para o banco de dados");

    Redirect::to("/cliente").into_response()
}
