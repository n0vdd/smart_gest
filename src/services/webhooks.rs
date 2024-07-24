use std::sync::Arc;

use axum::{http::{self}, response::IntoResponse, Extension, Json};
use chrono::{Datelike,  Local};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, PgPool};
use time::macros::format_description;
use tracing::{debug, error};

use crate::{handlers::clients::{Cliente, ClienteDto}, services::nfs::gera_nfs};

///!this is the api key for the sandbox, it should be set on the env for production
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
         if !check_if_payment_exists(&webhook_data.id, "payment_confirmed", &*pool).await.expect("Erro ao checar se o pagamento ja existe") {
            save_payment_confirmed(&pool, &webhook_data, format).await.expect("Erro ao salvar pagamento confirmado");
         }
         http::StatusCode::OK
      }, 

      Event::PaymentReceived => {
      //Gero nota fiscal ao confirmar o pagamento e cancela ela ao receber refund
         //TODO tenho que gerar nota fiscal quando o pagamento e recebido
         match webhook_data.payment_data.billing_type {
            BillingType::Boleto | BillingType::Pix | BillingType::CreditCard => {
               //TODO gerar nota fiscal de servico
               debug!("Gerando nota fiscal de servico para cliente: {:?}", webhook_data.payment_data.customer);
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
         if !check_if_payment_exists(&webhook_data.id, "payment_received", &*pool).await.expect("Erro ao checar se o pagamento ja existe") {
            save_payment_received(&pool, &webhook_data, format).await.expect("Erro ao salvar pagamento recebido");
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

pub async fn add_cliente_to_asaas(cliente:&ClienteDto) -> Result<(), anyhow::Error> {
   let client = reqwest::Client::new();
   
   let url = format!("https://sandbox.asaas.com/api/v3/customers/");

   //Pega uma  lista com todos os clientes
   client.get(&url).header("access_token",API_KEY)
      .header("accept", "application/json")
      .header("user-agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
      .send().await.map(|response| {
         debug!("Cliente Response: {:?}", response);
         response
      }).map_err(|e | {
         error!("Failed to fetch clients: {:?}", e);
         anyhow::anyhow!("Erro ao buscar clientes do sistema asaas")
      })?
      .json::<CustomerList>().await.map(|response| {
         debug!("Clientes em json: {:?}", response);
         response
      }).map_err(|e| {
         error!("Failed to parse clients: {:?}", e);
         anyhow::anyhow!("Erro ao realizar parse dos clientes")
      })?
      //Checa se o nome do cliente ja esta no sistema do asaas 
      .data.iter().find(|cl| cl.name == cliente.nome).map(|name| {
         debug!("Cliente ja existe: {:?}",name);
         //Caso o cliente ja exista nao ha necessidade de crialo
         return
      });

   let post_cliente = ClientePost {
      name: cliente.nome.clone(),
      email: cliente.email.clone(),
      cpf_cnpj: cliente.cpf_cnpj.clone(),
      mobile_phone: cliente.telefone.clone()
   };


   //Envia o cliente caso ele nao exista
   client.post(&url).header("access_token",API_KEY)
      .header("accept", "application/json")
      .header("content-type", "application/json")
      .header("user-agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
      .json(&post_cliente).send().await.map_err(|e| {
         error!("Failed to post client: {:?}", e);
         anyhow::anyhow!("Erro ao enviar cliente para sistema do asaas")
      })?;

   Ok(())
}

async fn check_if_payment_exists(id: &str, table: &str, pool: &PgPool) -> Result<bool,anyhow::Error> {
   //All payments have a event_id related to it
   let query_str = format!("SELECT event_id FROM {} WHERE event_id = $1", table);

   //Use the query to check if the event_id already exists in the table passed 
   let event_id = query(&query_str)
      .bind(id)
      //fetch_optional will return None if the query returns no results
      .fetch_optional(pool).await
      .map_err(|e| {
         error!("Failed to fetch payment: {:?}", e);
         anyhow::anyhow!("Erro ao buscar pagamento no banco de dados") 
      })?;

   //If the event_id exists, return true(is some)
   Ok(event_id.is_some())
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
      .header("accept", "application/json")
      .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
      .send()
      .await.map_err(|e| {
         error!("Failed to fetch client: {:?}", e);
         anyhow::anyhow!("Erro ao buscar cliente no sistema asaas")
      })?.json::<ClienteApi>().await.map_err(|e| {
         error!("Failed to parse client: {:?}", e);
         anyhow::anyhow!("Erro ao realizar parde do cliente vindo do sistema asaas")
      })?;

   debug!("Cliente: {:?}",client);

   //Confere se existe algum cliente com o cpf_cnpj
   let cliente = query_as!(Cliente,
      "SELECT * FROM clientes WHERE cpf_cnpj = $1",
      client.cpf_cnpj
   ).fetch_optional(&*pool).await.map_err(|e| {
      error!("Failed to fetch client: {:?}", e);
      anyhow::anyhow!("Erro ao buscar cliente no banco de dados")
   })?;

   //If the client does not exist, return an error
   if cliente.is_none() {
      Err(anyhow::anyhow!("Cliente nao encontrado"))
   } else {
      Ok(cliente.unwrap())
   }
}

///!This code is called only after a check if the event if exists on the db
async fn save_payment_received(pool:&PgPool,webhook_data: &Payload,format: &[time::format_description::BorrowedFormatItem<'_>]) -> Result<(),anyhow::Error>{
   if let Some(confirmed_data) = &webhook_data.payment_data.confirmed_date {
      // Format time for PostgreSQL
      let data = time::Date::parse(confirmed_data, format).map_err(|e| {
         error!("Failed to parse date: {:?}", e);
         anyhow::anyhow!("Erro ao tranformar string em uma data")
      })?;

      //Check if there is a payment already confirmed at the same time
      //?is this a good check?, i have the code for checkig if the payment exists in the db
      //what does this check does?, get the payment confirmed from the payment_confirmed date
      //BUG this could be a problem if the payment_date from payment_received is not the same as the payment_confirmed date 
      let payment_confirmed = query!(
         "SELECT id FROM payment_confirmed WHERE payment_date = $1",
         data
      )
      .fetch_optional(&*pool)
      .await.map_err(|e| {
         error!("Failed to fetch payment: {:?}", e);
         anyhow::anyhow!("Erro ao buscar pagamento no banco dados")
      })?;

      //Gets the id of the payment_confirmed if it exists
      if let Some(confirmed_id) = payment_confirmed {
         if let Some(data) = &webhook_data.payment_data.payment_date {
            // Format time for PostgreSQL
            let data = time::Date::parse(data, format).map_err(|e| {
               error!("Failed to parse date: {:?}", e);
               anyhow::anyhow!("Erro ao fazer parse de uma string em uma data")
            })?;

            //Save the payment_received in the db,linkink it to the payment_confirmed id
            query!(
               "INSERT INTO payment_received (event_id, payment_confirmed, payment_date) VALUES ($1, $2, $3)",
                  webhook_data.id,
                  confirmed_id.id,
                  data
            ).execute(&*pool)
            .await.map_err(|e| {
               error!("erro ao salvar pagamento {:?}", e);
               anyhow::anyhow!("Erro ao salvar pagamento recebido ao banco de dados")
            })?;
         }
      }
   }
   Ok(())
}

///!This code is called only after a check if the event if exists on the db
async fn save_payment_confirmed(pool:&PgPool,webhook_data: &Payload,format: &[time::format_description::BorrowedFormatItem<'_>]) -> Result<(),anyhow::Error>{
   debug!("Salvando pagamento confirmado: {:?}", webhook_data);
   //TODO maybe should check the event_id before saving it?

   //find the cliente from the asaas api
   let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool).await.map_err(|e| {
      error!("Failed to fetch client: {:?}", e);
      anyhow::anyhow!("Erro ao buscar cliente no sistema asaas")
   })?;

   // Format time for PostgreSQL
   if let Some(data) = &webhook_data.payment_data.payment_date {
      let data = sqlx::types::time::Date::parse(data, format).map_err(|e| {
         error!("Failed to parse date: {:?}", e);
         anyhow::anyhow!("Erro ao realizar parse de uma string em uma data")
      })?;

      //Save the payment_confirmed in the db
      query!(
         "INSERT INTO payment_confirmed (event_id, cliente_id, payment_date) VALUES ($1, $2, $3)",
            webhook_data.id,
            cliente.id,
            data
      ).execute(&*pool)
      .await.map_err(|e| {
         error!("Failed to save payment: {:?}", e);
         anyhow::anyhow!("Erro ao salvar pagamento confirmado no banco de dados")
      })?;
   }

   Ok(())
}

//? will receive webhook from payment gateway when payment is denied
//maybe it could be used to block client in radius aswell

