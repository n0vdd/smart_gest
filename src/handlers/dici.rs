use std::sync::Arc;

use anyhow::Context;
use axum::{response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use chrono::{Datelike, Local };
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, PgPool};
use time::{macros::format_description, Date, PrimitiveDateTime};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{error,debug};

use super::clients::{fetch_tipo_clientes_before_date,  TipoPessoa};

const CNPJ:&str = "48530335000148"; // Hardcoded CNPJ
const COD_IBGE:&str = "3106200"; // Hardcoded COD_IBGE

//#[derive(Template)]
//#[template(path = "dici_list.html")]
struct DiciTemplate {
  reference_date: Vec<String>,
  dicis: Vec<Dici>,
}

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
  let dicis = fetch_all_dici_records(&pool).await.expect("Erro ao pegar dicis da base de dados");

  let mut tera = tera::Tera::new("templates/**").map_err(|e| {
    error!("Failed to compile templates: {:?}", e);
  }).expect("Failed to compile templates");
  tera.add_template_file("templates/dici_list.html", Some("dici list")).map_err(|e| {
    error!("Failed to add template file: {:?}", e);
  }).expect("Failed to add template file");

  let mut context = tera::Context::new(); 
  context.insert("dicis", &dicis);

  let template = tera.render("dici list", &context).map_err(|e| {
    error!("Failed to render template: {:?}", e);
  }).expect("Failed to render template");

  Html(template)
}

#[derive(Deserialize)]
pub struct GenerateDiciForm {
  pub reference_date: String,
}

#[derive(Deserialize,Serialize,Debug)]
struct Dici {
  id: i32,
  reference_date: Date,
  created_at: Option<PrimitiveDateTime> 
}

async fn fetch_all_dici_records(pool: &PgPool) -> Result<Vec<Dici>,anyhow::Error> {
  //Fetch data that will be displayed on the DICI list
  let dicis = query_as!(Dici,
      "SELECT id,reference_date,created_at FROM dici ORDER BY reference_date DESC"
  )
  .fetch_all(pool)
  .await.map_err(|e| {
      error!("Failed to fetch DICI records: {:?}", e);
      anyhow::anyhow!("Erro ao recuperar dicis da base de dados")
  })?;

  Ok(dicis)
}

pub async fn generate_dici(Extension(pool): Extension<Arc<PgPool>>,Form(form): Form<GenerateDiciForm> ) -> impl IntoResponse {
  let date_format= format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
  let rf = format!("{}_{}_{}", Local::now().day(),form.reference_date,Local::now().time());
  debug!("Date struct for comparing with timestamp in db: {:?}", rf);
  let reference_date = time::PrimitiveDateTime::parse(&rf, &date_format).expect("Failed to parse reference date");

  //Pega clientes criados antes da data de referencia
  let clients = fetch_tipo_clientes_before_date(&pool,reference_date).await.expect("Falha ao pegar clientes antes da data de referencia");

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
  }).expect("Failed to create file");

  // Write CSV content to file
  file.write_all(csv_content.as_bytes()).await.map_err(|e| {
    error!("Failed to write to file: {:?}", e);
    //should not save dici to the db if the file was not writen
    //couldl generate incositencies in the data,should not stop all the code
    //TODO should have a modal for displaying errors
    return Html("<p>Falha ao salvar arquivo</p>".to_string());
  }).expect("Failed to write to file");

  //Make another date format to save in the db
  let date_format= format_description!("[day]_[month]_[year]");
  let rf = format!("{}_{}", Local::now().day(),form.reference_date);  

  //Parse the reference date for the db
  let reference_date = time::Date::parse(&rf, &date_format).expect("Failed to parse reference date");
  debug!("Reference Date struct: {:?}", reference_date);

  // Save the DICI to the database
  query!(
      "INSERT INTO dici (path,reference_date) VALUES ($1,$2)",
      path, reference_date
  )
  .execute(&*pool)
  .await.map_err(|e| {
      error!("Failed to save DICI record to the database: {:?}", e);
      e
  }).expect("Failed to save DICI record to the database");

  //Volta a listagem de dicis(pode dar problema dependendo do htmx)
  Redirect::to("/financeiro/dici")
}


pub async fn generate_dici_month_year(pool: &PgPool, month: u32, year: i32) -> Result<(),anyhow::Error> {
  // Date handling
  let date_format = format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
  let rf = format!("{}_{}_{}", Local::now().day(), month, year);
  debug!("Date struct for comparing with timestamp in db: {:?}", rf);
  let reference_date = time::PrimitiveDateTime::parse(&rf, &date_format).expect("Failed to parse reference date");

  let clients = fetch_tipo_clientes_before_date(pool, reference_date)
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
  let reference_date = Date::parse(&rf, &date_format).expect("Failed to parse reference date");
  debug!("Reference Date struct: {:?}", reference_date);

  sqlx::query!(
      "INSERT INTO dici (path, reference_date) VALUES ($1, $2)",
      path,
      reference_date
  )
  .execute(pool)
  .await
  .context("Failed to save DICI record to the database")?;

  Ok(())
}