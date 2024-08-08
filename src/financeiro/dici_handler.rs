use std::sync::Arc;

use anyhow::Context;
use axum::{response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use chrono::{Datelike, Local };
use sqlx::PgPool;
use time::{macros::format_description, Date };
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{error,debug};

use crate::{clientes::{cliente::fetch_tipo_clientes_before_date_for_dici, cliente_model::TipoPessoa}, financeiro::{dici::save_dici, dici_model::DiciDto}, TEMPLATES};

use super::{dici::find_all_dicis, dici_model::GenerateDiciForm};


//TODO get this from the provedor table
const CNPJ:&str = "48530335000148"; // Hardcoded CNPJ
//this data should be obrigatory for the provedor to generate nota fiscal
//TODO get this from the provedor table
//?should be on provedor or dici config?provedor for simplicity
//TODO do a check if this data exists before generating the DICI
//if not returns a error
const COD_IBGE:&str = "3106200"; // Hardcoded COD_IBGE


pub async fn show_dici_list(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
  let current_month = Local::now().month();
  let current_year = Local::now().year();

  let mut reference_date = (1..=current_month)
      .map(|month| format!("{:02}_{}", month, current_year))
      .collect::<Vec<String>>();

  let mut past_reference_date = (1..=12)
      .map(|month| format!("{:02}_{}", month, current_year - 1))
      .collect::<Vec<String>>();

  past_reference_date.append(&mut reference_date);

  // Fetch existing DICI records from the database
  let dicis = find_all_dicis(&pool).await.map_err(|e|
    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
  ).expect("Failed to fetch DICI records");

  let mut context = tera::Context::new(); 
  context.insert("dicis", &dicis);
  context.insert("reference_date", &past_reference_date);

  match TEMPLATES.render("financeiro/dici_list.html", &context) {
    Ok(template) => Html(template).into_response(),

    Err(e) => {
      error!("Failed to render DICI list template: {:?}", e);
      (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to render DICI list template").into_response()
    }
  }

}

pub async fn generate_dici(Extension(pool): Extension<Arc<PgPool>>,Form(form): Form<GenerateDiciForm> ) -> impl IntoResponse {
  let date_format= format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
  let rf = format!("{}_{}_{}", Local::now().day(),form.reference_date,Local::now().time());
  debug!("Date struct for comparing with timestamp in db: {:?}", rf);
  let reference_date = time::PrimitiveDateTime::parse(&rf, &date_format).expect("Failed to parse reference date");

  //Pega clientes criados antes da data de referencia
  let clients = fetch_tipo_clientes_before_date_for_dici(&pool,reference_date).await.map_err(|e|
    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
  )
  .expect("Falha ao pegar clientes antes da data de referencia");

  let mut pfs: Vec<TipoPessoa> = Vec::new();
  let mut pjs: Vec<TipoPessoa> = Vec::new();

  //Popula as duas listas para se ter a quantidade de cada tipo de cliente achado antes da data de referencia
  for cliente in clients {
    match cliente {
      TipoPessoa::PessoaFisica => pfs.push(cliente),
      TipoPessoa::PessoaJuridica => pjs.push(cliente)
    }
  }

  debug!("Pessoas Fisicas: {:?}", pfs);
  debug!("Pessoas Juridicas: {:?}", pjs);


  //Formatacao para nao haver erros ao salvar no sistema de arquivos
  let filename = format!("dici_{}.csv", form.reference_date);
  let path = format!("dicis/{}", filename);

  //Separao o ano e o mes da referencia para colocar no csv
  let mut splits = form.reference_date.split("_");
  let month = splits.nth(0).expect("falhar ao pegar mes de referencia");
  //debug!("Month: {:?}", month);
  //debug!("Splits: {:?}", splits);
  let year = splits.nth(0).expect("falhar ao pegar ano de referencia");

  //Preparo os headers
  let mut csv_content = String::from("CNPJ;ANO;MES;COD_IBGE;TIPO_CLIENTE;TIPO_ATENDIMENTO;TIPO_MEIO;TIPO_PRODUTO;TIPO_TECNOLOGIA;VELOCIDADE;ACESSOS\n");

  //funcao que coloca os clientes no csv de acordo com o tipo de cliente
  let append_client_count = |clients: Vec<TipoPessoa>, tipo_cliente: &str, csv_content: &mut String| {
      if !clients.is_empty() {
          let client_count = clients.len();
          let row = format!(
              "{};{};{};{};{};URBANO;fibra;internet;FTTB;200;{}\n",
              CNPJ,
              year,
              month,
              COD_IBGE,
              tipo_cliente,
              client_count
          );
          csv_content.push_str(&row);
      }
  };

  // Append PF clients
  append_client_count(pfs, "PF", &mut csv_content);

  // Append PJ clients
  append_client_count(pjs, "PJ", &mut csv_content);

  //creathe the dici file
  let mut file = File::create(&path).await.map_err(|e| {
    error!("Failed to create file: {:?}", e);
    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to create dici file".to_string());
  }).expect("Failed to create file");

  // Write CSV content to file
  file.write_all(csv_content.as_bytes()).await.map_err(|e| {
    error!("Failed to write to file: {:?}", e);
    //should not save dici to the db if the file was not writen
    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to dici file".to_string());
  }).expect("Failed to write to file");

  //Make another date format to save in the db
  let date_format= format_description!("[day]_[month]_[year]");
  let rf = format!("{}_{}", Local::now().day(),form.reference_date);  

  //Parse the reference date for the db
  let reference_date = time::Date::parse(&rf, &date_format).expect("Failed to parse reference date");
  debug!("Reference Date struct: {:?}", reference_date);
  
  let dici = &DiciDto {
    path,
    reference_date
  };

  match save_dici(&pool, dici).await {
    Ok(_) => Redirect::to("/dici").into_response(),

    Err(e) => {
      error!("Failed to save DICI record to the database: {:?}", e);
      (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to save DICI record to the database".to_string()).into_response()
    }
  }

}


pub async fn generate_dici_month_year(pool: &PgPool, month: u32, year: i32) -> Result<(),anyhow::Error> {
  // Date handling
  let date_format = format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
  let rf = format!("{}_{}_{}", Local::now().day(), month, year);
  debug!("Date struct for comparing with timestamp in db: {:?}", rf);
  let reference_date = time::PrimitiveDateTime::parse(&rf, &date_format).context("Failed to parse reference date")?;

  let clients = fetch_tipo_clientes_before_date_for_dici(pool, reference_date)
      .await
      .context("Falha ao pegar clientes antes da data de referencia")?;

  let mut pfs: Vec<TipoPessoa> = Vec::new();
  let mut pjs: Vec<TipoPessoa> = Vec::new();

  for cliente in clients {
      match cliente {
          TipoPessoa::PessoaFisica => pfs.push(cliente),
          TipoPessoa::PessoaJuridica => pjs.push(cliente),
      }
  }

  debug!("Pessoas Fisicas: {:?}", pfs);
  debug!("Pessoas Juridicas: {:?}", pjs);

  let filename = format!("dici_{}_{}.csv", year, month);
  let path = format!("dicis/{}", filename);

  let mut csv_content = String::from("CNPJ;ANO;MES;COD_IBGE;TIPO_CLIENTE;TIPO_ATENDIMENTO;TIPO_MEIO;TIPO_PRODUTO;TIPO_TECNOLOGIA;VELOCIDADE;ACESSOS\n");

  let append_client_count = |clients: Vec<TipoPessoa>, tipo_cliente: &str, csv_content: &mut String| {
      if !clients.is_empty() {
          let client_count = clients.len();
          let row = format!(
              "{};{};{};{};{};URBANO;fibra;internet;FTTB;200;{}\n",
              CNPJ,
              year,
              month,
              COD_IBGE,
              tipo_cliente,
              client_count
          );
          csv_content.push_str(&row);
      }
  };

  append_client_count(pfs, "PF", &mut csv_content);
  append_client_count(pjs, "PJ", &mut csv_content);

  let mut file = File::create(&path)
      .await
      .context("Failed to create file")?;
  file.write_all(csv_content.as_bytes())
      .await
      .context("Failed to write to file")?;

  let date_format = format_description!("[day]_[month]_[year]");
  let rf = format!("{}_{}_{}", Local::now().day(), month, year);
  let reference_date = Date::parse(&rf, &date_format).context("Failed to parse reference date")?;
  debug!("Reference Date struct: {:?}", reference_date);

  let dici = &DiciDto {
      path,
      reference_date,
  };

  match save_dici(&pool, &dici).await {
    Ok(_) => Ok(()),

    Err(e) => {
      error!("Failed to save DICI record to the database: {:?}", e);
      Err(anyhow::anyhow!("Falha ao salvar DICI no banco de dados"))
    }
  }

}