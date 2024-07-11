use std::sync::Arc;

use askama::Template;
use axum::{response::IntoResponse, Extension};
use chrono::{Datelike, Local };
use sqlx::{query, query_as, PgPool};
use time::{Date, PrimitiveDateTime};
use tracing::error;


#[derive(Template)]
#[template(path = "dici_list.html")]
struct DiciTemplate {
  years: Vec<i32>,
  months: Vec<u32>,
  dicis: Vec<Dici>,
}

pub async fn show_dici_list(Extension(pool):Extension<Arc<PgPool>>) -> impl IntoResponse {
  let month = chrono::Local::now().month();
  //?This will return all the months until the actual one
  let months = chrono::Months::new(month);
  //?this will return all the months
  let months = chrono::Months::new(12);
  let year = Local::now().year();
  let past_yerar = year - 1;
  let dici = fetch_dici_records(&pool).await;

  //let template = DiciTemplate { year, month };
}

struct Dici {
  reference_date: Date,
  created_at: Option<PrimitiveDateTime> 
}

async fn fetch_dici_records(pool: &PgPool) -> Vec<Dici> {
  // Query to fetch DICI records from the database
  query_as!(Dici,
      "SELECT reference_date,created_at FROM dici ORDER BY reference_date DESC"
  )
  .fetch_all(pool)
  .await.map_err(|e| {
      error!("Failed to fetch DICI records: {:?}", e);
      e
  }).expect("Failed to fetch DICI records")
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


