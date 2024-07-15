use std::sync::Arc;

use askama:: Template;
use axum::{response::{Html, IntoResponse, Redirect}, Extension};
use axum_extra::extract::Form;
use chrono::{Datelike, Local, SubsecRound };
use phonenumber::country::Id::YE;
use serde::Deserialize;
use sqlx::{query_as, PgPool};
use time::{macros::format_description, Date, PrimitiveDateTime};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{error,debug};
use tracing_subscriber::field::debug;

use super::clients::{fetch_tipo_clientes_before_date,  TipoPessoa};

#[derive(Template)]
#[template(path = "dici_list.html")]
struct DiciTemplate {
  reference_date: Vec<String>,
  dicis: Vec<Dici>,
}

pub async fn show_dici_list(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
  let current_year = Local::now().year();
  let month = Local::now().month();

  let mut reference_date = (1..=12)
      .map(|month| format!("{:02}_{}", month, current_year))
      .collect::<Vec<String>>();
  
  let mut past_reference_date = (1..=12)
      .map(|month| format!("{:02}_{}", month, current_year - 1))
      .collect::<Vec<String>>();

  reference_date.append(&mut past_reference_date);


  // Fetch existing DICI records from the database
  let dicis = fetch_dici_records(&pool).await;

  let template = DiciTemplate { reference_date, dicis,}.render()
  .map_err(|e| {
    error!("Failed to render DICI list template: {:?}", e);
    e
  }).expect("Failed to render DICI list template");

  Html(template)
}

#[derive(Deserialize)]
pub struct GenerateDiciForm {
  reference_date: String,
}

struct Dici {
  id: i32,
  reference_date: Date,
  created_at: Option<PrimitiveDateTime> 
}

async fn fetch_dici_records(pool: &PgPool) -> Vec<Dici> {
  // Query to fetch DICI records from the database
  query_as!(Dici,
      "SELECT id,reference_date,created_at FROM dici ORDER BY reference_date DESC"
  )
  .fetch_all(pool)
  .await.map_err(|e| {
      error!("Failed to fetch DICI records: {:?}", e);
      e
  }).expect("Failed to fetch DICI records")
}

pub async fn generate_dici(Extension(pool): Extension<Arc<PgPool>>,Form(form): Form<GenerateDiciForm> ) -> impl IntoResponse {
  // Logic to generate DICI and save it to the file system and database



  //Formato para comparar com o created_t na db 
  let date_format= format_description!("[day]_[month]_[year]_[hour]:[minute]:[second].[subsecond]");
  let rf = format!("{}_{}_{}", Local::now().day(),form.reference_date,Local::now().time());
  debug!("Date struct for comparing with timestamp in db: {:?}", rf);
  let reference_date = time::PrimitiveDateTime::parse(&rf, &date_format).expect("Failed to parse reference date");
  let clients = fetch_tipo_clientes_before_date(&pool,reference_date).await;

  let mut pfs: Vec<TipoPessoa> = Vec::new();
  let mut pjs: Vec<TipoPessoa> = Vec::new();

  for cliente in clients {
    match cliente {
      TipoPessoa::PessoaFisica => pfs.push(cliente),
      TipoPessoa::PessoaJuridica => pjs.push(cliente)
    }
  }
  debug!("Pessoa Fisica: {:?}", pfs);
  debug!("Pessoa Juridica: {:?}", pjs);


  //Formatacao para nao haver erros ao salvar no sistema de arquivos
  let filename = format!("dici_{}.csv", form.reference_date);
  let path = format!("dicis/{}", filename);

  //Pegando dados para colocar no csv
  let mut splits = form.reference_date.split("_");
  let month = splits.nth(0).expect("falhar ao pegar mes de referencia");
  //debug!("Month: {:?}", month);
  //debug!("Splits: {:?}", splits);
  let year = splits.nth(0).expect("falhar ao pegar ano de referencia");

 
  // Generate CSV content
  let mut csv_content = String::from("CNPJ;ANO;MES;COD_IBGE;TIPO_CLIENTE;TIPO_ATENDIMENTO;TIPO_MEIO;TIPO_PRODUTO;TIPO_TECNOLOGIA;VELOCIDADE;ACESSOS\n");
  let cnpj = "48530335000148"; // Hardcoded CNPJ
  let cod_ibge = "3106200"; // Hardcoded COD_IBGE

  let append_client_count = |clients: Vec<TipoPessoa>, tipo_cliente: &str, csv_content: &mut String| {
      if !clients.is_empty() {
          let client_count = clients.len();
          let row = format!(
              "{};{};{};{};{};URBANO;fibra;internet;FTTB;200;{}\n",
              cnpj,
              year,
              month,
              cod_ibge,
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

  // Write CSV content to file
  let mut file = File::create(&path).await.expect("Failed to create file");
  file.write_all(csv_content.as_bytes()).await.expect("Failed to write to file");

  let date_format= format_description!("[day]_[month]_[year]");
  let rf = format!("{}_{}", Local::now().day(),form.reference_date);  
  //Parse the reference date for the db
  let reference_date = time::Date::parse(&rf, &date_format).expect("Failed to parse reference date");
  debug!("Reference Date struct: {:?}", reference_date);
  // Save the DICI to the database
  sqlx::query!(
      "INSERT INTO dici (path,reference_date) VALUES ($1,$2)",
      path, reference_date
  )
  .execute(&*pool)
  .await.map_err(|e| {
      error!("Failed to save DICI record to the database: {:?}", e);
      e
  }).expect("Failed to save DICI record to the database");

  Redirect::to("/financeiro/dici")
}



//TODO gera dici csv
//ter uma form deixando voce decidir o mes e o ano
//checa todos os clientes que ja haviam sido cadastrados ate esse periodo
//gera o dici.csv, depois so mandar para a anatel
//temos o template funcionando, dici_template.csv
/* 
CNPJ;ANO;MES;COD_IBGE;TIPO_CLIENTE;TIPO_ATENDIMENTO;TIPO_MEIO;TIPO_PRODUTO;TIPO_TECNOLOGIA;VELOCIDADE;ACESSOS
48530335000148;2024;05;3106200;PJ;URBANO;fibra;internet;FTTB;100;1
*/

//tem que usar um utf-8 especifico,

/*
ANATEL - Agência Nacional de Telecomunicações
Leiaute: Acessos - SCM
Formato do arquivo: CSV delimitado por ";" 
Codificação esperada do arquivo: UTF-8 BOM
Cabeçalho (na primeira linha):
CNPJ;ANO;MES;COD_IBGE;TIPO_CLIENTE;TIPO_ATENDIMENTO;TIPO_MEIO;TIPO_PRODUTO;TIPO_TECNOLOGIA;VELOCIDADE;ACESSOS

Além do cabeçalho, é necessário informar pelo menos uma linha com registros.
Detalhe dos campos:
Posição Nome                Descrição                                          Tipo                                 Obrigatório
1       CNPJ                CNPJ da empresa prestadora do serviço com exatamen Texto com limitação de caracteres    Sim
                            te 14 dígitos, incluindo zeros à esquerda.                                              
2       ANO                 Ano referente aos dados, com 4 dígitos numéricos.  Número inteiro                       Sim
3       MES                 Mês referente aos dados, com até 2 dígitos numéric Número inteiro                       Sim
                            os.                                                                                     
4       COD_IBGE            Código IBGE de identificação do Município, com 7 d Número inteiro                       Sim
                            ígitos, onde se localizam os acessos em serviço in                                      
                            dicados.                                                                                
5       TIPO_CLIENTE        Identificação do grupo de acessos, se Pessoa Juríd Texto com limitação de caracteres    Sim
                            ica (PJ), Pessoa Física (PF) e Uso próprio (UP).                                        
6       TIPO_ATENDIMENTO    Classificação do conjunto de acessos se URBANO ou  Texto com limitação de caracteres    Sim
                            RURAL.                                                                                  
7       TIPO_MEIO           Meio de acesso usado para conectar o conjunto de a Texto com limitação de caracteres    Sim
                            cessos, podendo ser "cabo_coaxial", "cabo_metalico                                      
                            ", "fibra", "radio", "satelite".                                                        
8       TIPO_PRODUTO        Tipo de uso ao qual se destina o conjunto de aceso Texto com limitação de caracteres    Sim
                            s, podendo ser "internet", "linha_dedicada", "m2m"                                      
                              ou "outros".                                                                          
9       TIPO_TECNOLOGIA     Tecnologia empregada para dar conectividade ao con Texto com limitação de caracteres    Sim
                            junto de acessos, segundo a tabela de tecnologias                                       
                            publicada pela Agência.                                                                 
10      VELOCIDADE          Velocidade contratada (plano de serviço) pelo grup Texto com limitação de caracteres    Sim
                            o de acessos, em Mbps. É requerido um valor maior                                       
                            do que zero. IMPORTANTE: Esse campo aceita apenas                                       
                            a vírgula como separador decimal e não admite a pr                                      
                            esença de ponto, nem mesmo como separador de milha                                      
                            r.                                                                                      
11      ACESSOS             Número de acessos do conjunto representado no pres Número inteiro                       Sim
                            ente registro.                                                                          



Posição
1
Nome
CNPJ
Descrição
CNPJ da empresa prestadora do serviço com exatamente 14 dígitos, incluindo zeros à esquerda.
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(14)
Obrigatório
Sim
Posição
2
Nome
ANO
Descrição
Ano referente aos dados, com 4 dígitos numéricos.
Tipo do Dado (Tamanho)
Número inteiro(4)
Obrigatório
Sim
Posição
3
Nome
MES
Descrição
Mês referente aos dados, com até 2 dígitos numéricos.
Tipo do Dado (Tamanho)
Número inteiro(2)
Obrigatório
Sim
Posição
4
Nome
COD_IBGE
Descrição
Código IBGE de identificação do Município, com 7 dígitos, onde se localizam os acessos em serviço indicados.
Tipo do Dado (Tamanho)
Número inteiro(7)
Obrigatório
Sim
Posição
5
Nome
TIPO_CLIENTE
Descrição
Identificação do grupo de acessos, se Pessoa Jurídica (PJ), Pessoa Física (PF) e Uso próprio (UP).
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(2)
Obrigatório
Sim
Posição
6
Nome
TIPO_ATENDIMENTO
Descrição
Classificação do conjunto de acessos se URBANO ou RURAL.
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(6)
Obrigatório
Sim
Posição
7
Nome
TIPO_MEIO
Descrição
Meio de acesso usado para conectar o conjunto de acessos, podendo ser "cabo_coaxial", "cabo_metalico", "fibra", "radio", "satelite".
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(20)
Obrigatório
Sim
Posição
8
Nome
TIPO_PRODUTO
Descrição
Tipo de uso ao qual se destina o conjunto de acesos, podendo ser "internet", "linha_dedicada", "m2m"  ou "outros".
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(30)
Obrigatório
Sim
Posição
9
Nome
TIPO_TECNOLOGIA
Descrição
Tecnologia empregada para dar conectividade ao conjunto de acessos, segundo a tabela de tecnologias publicada pela Agência.
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(25)
Obrigatório
Sim
Posição
10
Nome
VELOCIDADE
Descrição
Velocidade contratada (plano de serviço) pelo grupo de acessos, em Mbps. É requerido um valor maior do que zero. IMPORTANTE: Esse campo aceita apenas a vírgula como separador decimal e não admite a presença de ponto, nem mesmo como separador de milhar.
Tipo do Dado (Tamanho)
Texto com limitação de caracteres(31)
Obrigatório
Sim
Posição
11
Nome
ACESSOS
Descrição
Número de acessos do conjunto representado no presente registro.
Tipo do Dado (Tamanho)
Número inteiro(8)
Obrigatório
Sim
*/


