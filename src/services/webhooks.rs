use std::sync::Arc;

use axum::{http::{self}, response::IntoResponse, Extension, Json};
use chrono::{Datelike,  Local};
use serde::{Deserialize, Serialize};
use serde_json::ser;
use sqlx::{query, query_as, PgPool};
use time::{format_description::FormatItem, macros::format_description};
use tracing::{debug, error};

use crate::{handlers::clients::{Cliente, ClienteDto}, services::nfs::gera_nfs};

const API_KEY: &str = "$aact_YTU5YTE0M2M2N2I4MTliNzk0YTI5N2U5MzdjNWZmNDQ6OjAwMDAwMDAwMDAwMDAwODUzNzI6OiRhYWNoXzAzYTI4MDhmLWI0NmItNDliNC1hNTIwLTRkNWUzZDBjNTQxZg==";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BillingType {
   Boleto,
   CreditCard,
   Undefined,
   DebitCard,
   Transfer,
   Deposit,
   Pix,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Event {
   PaymentCreated,
   PaymentAwaitingRiskAnalysis,
   PaymentApprovedByRiskAnalysis,
   PaymentReprovedByRiskAnalysis,
   PaymentAuthorized,
   PaymentUpdated,
   PaymentConfirmed,
   PaymentReceived,
   PaymentCreditCardCaptureRefused,
   PaymentAnticipated,
   PaymentOverdue,
   PaymentDeleted,
   PaymentRestored,
   PaymentRefunded,
   PaymentPartiallyRefunded,
   PaymentRefundInProgress,
   PaymentReceivedInCashUndone,
   PaymentChargebackRequested,
   PaymentChargebackDispute,
   PaymentAwaitingChargebackReversal,
   PaymentDunningReceived,
   PaymentDunningRequested,
   PentBankSlipViewed,
   PaymentCheckoutViewed,
}

#[derive(Serialize, Deserialize, Debug)]
struct Payment {
   id: String,
   #[serde(rename = "dateCreated")]
   date_created: String,
   // sera usado para linkar ao cliente
   customer: String,
   #[serde(rename = "paymentDate")]
   payment_date: Option<String>,
   #[serde(rename = "confirmedDate")]
   confirmed_date: Option<String>,
   #[serde(rename = "billingType")]
   billing_type: BillingType,
   #[serde(rename = "netValue")]
   net_value: f32,

}

#[derive(Serialize,Deserialize,Debug)]
pub struct Payload {
      id: String,
      event: Event,
      #[serde(rename = "dateCreated")]
      date_created: String,
      #[serde(rename = "payment")]
      payment_data: Payment,
}

//todo dia 12 do mes os clientes que nao tiverem um pagamento confirmado serao desativados do servidor radius
//TODO gerar nota fiscal de servico apos receber pagamento
//TODO radius deveria checar todo dia 12 os clientes que nao tem um pagamente confirmado
pub async fn webhook_handler(
   Extension(pool):Extension<Arc<PgPool>>,Json(webhook_data):Json<Payload>) -> impl IntoResponse {
   debug!("Webhook data: {:?}", webhook_data);

   let format = format_description!("[year]-[month]-[day]");
   match webhook_data.event {
      Event::PaymentConfirmed => {
         // Check if the event already exists in payment_confirmed table
         if !check_if_payment_exists(&webhook_data.id, "payment_confirmed", &*pool).await {
            save_payment_confirmed(&pool, &webhook_data, format).await;
         }
         http::StatusCode::OK
      }, 

      Event::PaymentReceived => {
      //Gero nota fiscal ao confirmar o pagamento e cancela ela ao receber refund
         //TODO tenho que gerar nota fiscal quando o pagamento e recebido
         match webhook_data.payment_data.billing_type {
            BillingType::Boleto | BillingType::Pix | BillingType::CreditCard => {
               //TODO gerar nota fiscal de servico
               let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool).await.map_err(|e| {
                  error!("Failed to fetch client: {:?}", e);
                  e
               }).expect("Erro ao buscar cliente");
               gera_nfs(cliente,webhook_data.payment_data.net_value).await;
            },
            _ => {
               //TODO mandar algum aviso e logar, nao deveria nem chegar nesse flow
               //so vendemoos boleto, cartao de credito e pix
               error!("Tipo de pagamento nao suportado: {:?}", webhook_data.payment_data.billing_type);
            }
         }

         // Check if the event already exists in payment_received table
         if !check_if_payment_exists(&webhook_data.id, "payment_received", &*pool).await {
            save_payment_received(&pool, &webhook_data, format).await;
         }

         http::StatusCode::OK
      },

      Event::PaymentRefunded => {
         //TODO cancelar nota fiscal de servico
         if Local::now().day() > 12 {
            // TODO: block client in radius
         }
         http::StatusCode::OK
      }, 

      Event::PaymentRefundInProgress => {
         if Local::now().day() > 12 {
            // TODO: block client in radius
         }
         http::StatusCode::OK
      },

      _ => {
         http::StatusCode::OK
      }
   }
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomerData {
   object: String,
   id: String,
   #[serde(rename = "dateCreated")]
   date_created: String,
   name: String,
   email: String,
   #[serde(rename = "cpfCnpj")]
   cpf_cnpj: String,
   #[serde(rename = "mobilePhone")]
   mobile_phone: String,
   deleted: bool,
}
#[derive(Serialize, Deserialize, Debug)]
struct CustomerList {
   object: String,
   #[serde(rename = "hasMore")]
   has_more: bool,
   #[serde(rename = "totalCount")]
   total_count: i32,
   limit: i32,
   offset: i32,
   data: Vec<CustomerData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientePost{
   name: String,
   email: String,
   #[serde(rename = "cpfCnpj")]
   cpf_cnpj: String,
   #[serde(rename = "mobilePhone")]
   mobile_phone:String
}

pub async fn add_cliente_to_asaas(cliente:&ClienteDto) {
   let client = reqwest::Client::new();
   
   let url = format!("https://sandbox.asaas.com/api/v3/customers/");

   client.get(&url).header("access_token",API_KEY)
      .send().await.expect("Erro ao enviar pedido para recuperar clientes")
      .json::<CustomerList>().await.expect("Erro ao receber clientes")
      .data.iter().find(|name| name.name == cliente.nome).map(|name| {
         debug!("Cliente ja existe: {:?}",name);
         return
      });

   let post_cliente = ClientePost {
      name: cliente.nome.clone(),
      email: cliente.email.clone(),
      cpf_cnpj: cliente.cpf_cnpj.clone(),
      mobile_phone: cliente.telefone.clone()
   };


   client.post(&url).header("access_token",API_KEY)
      .json(&post_cliente).send().await
      .expect("Erro ao enviar pedido para adicionar cliente");
}

async fn check_if_payment_exists(id: &str, table: &str, pool: &PgPool) -> bool {
   let query_str = format!("SELECT event_id FROM {} WHERE event_id = $1", table);
   let event_id = query(&query_str)
      .bind(id)
      .fetch_optional(pool)
      .await
      .map_err(|e| {
         error!("Failed to fetch payment: {:?}", e);
         e
      })
      .expect("Erro ao buscar pagamento");
   event_id.is_some()
}

#[derive(Serialize,Deserialize,Debug)]
struct ClienteApi {
   id: String,
   #[serde(rename = "cpfCnpj")]
   cpf_cnpj: String,
}

//TODO usar api key no ambiente
async fn find_api_cliente(id:&str,pool: &PgPool) -> Result<Cliente,anyhow::Error> {
   //TODO send a request to this url: https://sandbox.asaas.com/api/v3/customers/{id}
   //producao: https://www.asaas.com/api/v3/customers/{id}
   //get the cpfCnpj from the response and use it to find the cliente in the db
   let client = reqwest::Client::new()
      .get(format!("https://sandbox.asaas.com/api/v3/customers/{}",id))
      .header("access_token",API_KEY)
      .send()
      .await.map_err(|e| {
         error!("Failed to fetch client: {:?}", e);
         e
      })?;

   let cliente_api = client.json::<ClienteApi>().await.map_err(|e| {
      error!("Failed to fetch client: {:?}", e);
      e
   })?;

   debug!("Cliente: {:?}",cliente_api);

   let cliente = query_as!(Cliente,
      "SELECT * FROM clientes WHERE cpf_cnpj = $1",
      cliente_api.cpf_cnpj
   ).fetch_optional(&*pool).await.map_err(|e| {
      error!("Failed to fetch client: {:?}", e);
      e
   })?;

   if cliente.is_none() {
      Err(anyhow::Error::msg("Cliente nao encontrado"))
   } else {
      Ok(cliente.unwrap())
   }
}

async fn save_payment_received(pool:&PgPool,webhook_data: &Payload,format: &[time::format_description::BorrowedFormatItem<'_>]) {
   if let Some(confirmed_data) = &webhook_data.payment_data.confirmed_date {
      let data = time::Date::parse(confirmed_data, format).map_err(|e| {
         error!("Failed to parse date: {:?}", e);
         e
      }).expect("Erro ao tranformar string em uma data");

      let payment_confirmed = query!(
         "SELECT id FROM payment_confirmed WHERE payment_date = $1",
            data
      )
      .fetch_optional(&*pool)
      .await.map_err(|e| {
         error!("Failed to fetch payment: {:?}", e);
         e
      }).expect("Erro ao buscar pagamento");

      if let Some(confirmed_id) = payment_confirmed {
         if let Some(data) = &webhook_data.payment_data.payment_date {
            let data = time::Date::parse(data, format).map_err(|e| {
               error!("Failed to parse date: {:?}", e);
               e
            }).expect("Erro ao tranformar string em uma data");

            query!(
               "INSERT INTO payment_received (event_id, payment_confirmed, payment_date) VALUES ($1, $2, $3)",
                  webhook_data.id,
                  confirmed_id.id,
                  data
            ).execute(&*pool)
            .await.map_err(|e| {
               error!("erro ao salvar pagamento {:?}", e);
               e
            }).expect("Erro ao salvar pagamento");
         }
      }
   }
}

async fn save_payment_confirmed(pool:&PgPool,webhook_data: &Payload,format: &[time::format_description::BorrowedFormatItem<'_>]) {
   let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool).await.map_err(|e| {
      error!("Failed to fetch client: {:?}", e);
      e
   }).expect("Erro ao buscar cliente");

   // Format time for PostgreSQL
   if let Some(data) = &webhook_data.payment_data.payment_date {
      let data = sqlx::types::time::Date::parse(data, format).map_err(|e| {
         error!("Failed to parse date: {:?}", e);
         e
      }).expect("Erro ao tranformar string em uma data");

      query!(
         "INSERT INTO payment_confirmed (event_id, cliente_id, payment_date) VALUES ($1, $2, $3)",
            webhook_data.id,
            cliente.id,
            data
      ).execute(&*pool)
      .await.map_err(|e| {
         error!("Failed to save payment: {:?}", e);
         e
      }).expect("Erro ao salvar pagamento");
   }
}

//? will receive webhook from payment gateway when payment is denied
//maybe it could be used to block client in radius aswell

