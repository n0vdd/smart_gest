use std::sync::Arc;

use axum::{extract::Path, response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use radius::{create_radius_plano, PlanoRadiusDto};
use sqlx:: PgPool;
use tracing::error;

use crate::{financeiro::contrato::find_all_contrato_templates, TEMPLATES};


use super::{plano::{delete_plano_by_id, find_all_planos, find_plano_by_id, save_plano, update_plano_db}, plano_model::{Plano, PlanoDto, TipoPagamento}};

//Deleta o plano basead na id passada(pelo button de delete)
pub async fn delete_plano(Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<i32>)
    -> impl IntoResponse {
        match delete_plano_by_id(&pool, id).await {
            Ok(_) => Redirect::to("/plano").into_response(),

            Err(e) => {
                error!("Failed to delete plano: {:?}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
}

//Pega todos os planos criados na db
//popula uma template com eles
//retorna a listagem(template)
pub async fn list_planos(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    let planos = find_all_planos(&pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar planos na db");

    let mut context = tera::Context::new();
    context.insert("planos", &planos);

    let template = TEMPLATES.lock().await;
    match template.render("plano/plano_list.html", &context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render plano list template: {:?}", e);
            let erro = format!("Erro ao renderizar lista de planos {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
        }
    }
}

//Recebe os dados de edicao de um plano
//Atualiza o plano na db
//Retorna a listagem com todos os planos
pub async fn update_plano(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(plano): Form<Plano>,
) -> impl IntoResponse  {
    match update_plano_db(&pool, &plano).await {
        Ok(_) => Redirect::to("/plano").into_response(),

        Err(e) => {
            error!("Failed to update plano: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

//Recebe a id de um plano pelo button de editar
//Busca o plano na db
//Busca todas as templates de contrato na db(popular edit form tambem)
//Popula uma template com os dados do plano
//Exibe o formulario de exibicao populado para o usuario
pub async fn show_plano_edit_form(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<i32>
) -> impl IntoResponse {
    let plano = find_plano_by_id(&pool, id).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar plano na db"); 

    //Possivel selecionar qualquer uma das templates de contrato para ser usado pelo plano
    //Elas sao usadas ao criar um contrato para o cliente
    let contratos = find_all_contrato_templates(&pool).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar contratos na db"); 

    let mut context = tera::Context::new();
    context.insert("plano", &plano);
    context.insert("contratos", &contratos);

    let template = TEMPLATES.lock().await;
    match template.render("plano/plano_edit.html", &context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render plano edit template: {:?}", e);
            let erro = format!("Erro ao renderizar formulario de edicao de plano {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
        }
    }
}


//Acha todas as templates de contrato na db
//Popula a template de criacao de planos com as opcoes de contrato
//renderiza a template e a retorna para o usuario
pub async fn show_planos_form(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    //Usados para gerar o contrato do cliente posteriormente
    let contratos = find_all_contrato_templates(&pool).await.map_err(|e|
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao buscar contratos na db"); 

    let mut context = tera::Context::new();
    context.insert("contratos", &contratos);
    let pagamentos = vec![TipoPagamento::Boleto,TipoPagamento::CartaoCredito,TipoPagamento::Pix]; 
    context.insert("tipo_pagamento", &pagamentos);

    let template = TEMPLATES.lock().await;
    match template.render("plano/plano_add.html", &context) {
        Ok(template) => Html(template).into_response(),

        Err(e) => {
            error!("Failed to render plano add template: {:?}", e);
            let erro = format!("Erro ao renderizar formulario de criacao de plano {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, erro).into_response()
        }
    }
}

//Recebe os dados de um plano
//Salva o mesmo para a db
//Retorna a listagem de planos
pub async fn register_plano(
    Extension(pool): Extension<Arc<PgPool>>,
    Form(plano): Form<PlanoDto>,
) -> impl IntoResponse {
    let transaction = pool.begin().await.expect("Erro ao iniciar transacao");

    save_plano(&pool, &plano).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Erro ao salvar plano para o banco de dados");

    let radius_plano = PlanoRadiusDto {
        nome: plano.nome,
        velocidade_up: plano.velocidade_up,
        velocidade_down: plano.velocidade_down
    };

    create_radius_plano(radius_plano).await.map_err(|e| 
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
    ).expect("Failed to create radius plano");

    transaction.commit().await.expect("Erro ao commitar transacao");
    Redirect::to("/plano")
}
