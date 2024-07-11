use std::{net::{IpAddr, Ipv4Addr}, sync::Arc};

use axum::{extract::Request, http::{self, status::InvalidStatusCode}, response::IntoResponse, Extension, Json};
use chrono::{Datelike,  Local};
use once_cell::sync::Lazy;
use serde::{de, Deserialize, Serialize};
use sqlx::{pool, query, query_as, PgPool};
use time::macros::format_description;
use tracing::{debug, error};

use crate::handlers::clients::Cliente;


static ASSAS_IPS: Lazy<Vec<IpAddr>> = Lazy::new(|| {
   vec![
      IpAddr::V4(Ipv4Addr::new(52,67,12,206)),
      IpAddr::V4(Ipv4Addr::new(18,230,8,159)),
      IpAddr::V4(Ipv4Addr::new(54,94,136,112)),
      IpAddr::V4(Ipv4Addr::new(54,94,183,101)),
      IpAddr::V4(Ipv4Addr::new(54,207,175,46)),
      IpAddr::V4(Ipv4Addr::new(54,94,35,137)),
   ]
});


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
   value: f64,
   net_value: f64,
   original_value: Option<f64>,
   interest_value: Option<f64>,
   description: String,
   can_be_paid_after_due_date: bool,
   status: String,
   due_date: String,
   original_due_date: String,
   client_payment_date: Option<String>,
   invoice_url: String,
   invoice_number: String,
   external_reference: Option<String>,
   deleted: bool,
   anticipated: bool,
   anticipable: bool,
   credit_date: Option<String>,
   estimated_credit_date: Option<String>,
   transaction_receipt_url: Option<String>,
   nosso_numero: Option<String>,
   bank_slip_url: Option<String>,
   last_invoice_viewed_date: Option<String>,
   last_bank_slip_viewed_date: Option<String>,
   postal_service: bool,
   custody: Option<String>,
   refunds: Option<String>,
}


#[derive(Serialize,Deserialize,Debug)]
pub struct Payload {
      id: String,
      event: Event,
      #[serde(rename = "dateCreated")]
      date_created: String,
      payment_data: Payment,
}

pub async fn debug(req:Request) -> impl IntoResponse {
   debug!("Request: {:?}", req);
   http::StatusCode::OK
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
                  )
                  .execute(&*pool)
                  .await.map_err(|e| {
                     error!("Failed to save payment: {:?}", e);
                     e
                  }).expect("Erro ao salvar pagamento");
               }
            }

            //Gero nota fiscal ao confirmar o pagamento e cancela ela ao receber refund
            match webhook_data.payment_data.billing_type {
               BillingType::Boleto | BillingType::Pix | BillingType::CreditCard => {
                  //TODO gerar nota fiscal de servico
               },
               _ => {
                  //TODO mandar algum aviso e logar, nao deveria nem chegar nesse flow
                  //so vendemoos boleto, cartao de credito e pix
                  error!("Tipo de pagamento nao suportado: {:?}", webhook_data.payment_data.billing_type);
               }
            } 
            http::StatusCode::OK
         },
         Event::PaymentReceived => {
            // Check if the event already exists in payment_received table
            if !check_if_payment_exists(&webhook_data.id, "payment_received", &*pool).await {
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
                     .await
                     .map_err(|e| {
                        error!("Failed to fetch payment: {:?}", e);
                        e
                     })
                     .expect("Erro ao buscar pagamento");

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
                        )
                        .execute(&*pool)
                        .await
                        .map_err(|e| {
                           error!("erro ao salvar pagamento {:?}", e);
                           e
                        })
                        .expect("Erro ao salvar pagamento");
                     }
                  }
               }
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
      .header("access_token","sandbox")
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

/* this is the json received from the webhook
   "id": "evt_05b708f961d739ea7eba7e4db318f621&368604920",
   "event":"PAYMENT_RECEIVED",
   "dateCreated": "2024-06-12 16:45:03",
   "payment":{
      "object":"payment",
      "id":"pay_080225913252",
      "dateCreated":"2021-01-01",
      "customer":"cus_G7Dvo4iphUNk",
      "subscription":"sub_VXJBYgP2u0eO",  
         // somente quando pertencer a uma assinatura
      "installment":"2765d086-c7c5-5cca-898a-4262d212587c",
         // somente quando pertencer a um parcelamento
      "paymentLink":"123517639363",
         // identificador do link de pagamento
      "dueDate":"2021-01-01",
      "originalDueDate":"2021-01-01",
      "value":100,
      "netValue":94.51,
      "originalValue":null,
         // para quando o valor pago é diferente do valor da cobrança
      "interestValue":null,
      "nossoNumero": null,
      "description":"Pedido 056984",
      "externalReference":"056984",
      "billingType":"CREDIT_CARD",
      "status":"RECEIVED",
      "pixTransaction":null,
      "confirmedDate":"2021-01-01",
      "paymentDate":"2021-01-01",
      "clientPaymentDate":"2021-01-01",
      "installmentNumber": null,
      "creditDate":"2021-02-01",
      "custody": null,
      "estimatedCreditDate":"2021-02-01",
      "invoiceUrl":"https://www.asaas.com/i/080225913252",
      "bankSlipUrl":null,
      "transactionReceiptUrl":"https://www.asaas.com/comprovantes/4937311816045162",
      "invoiceNumber":"00005101",
      "deleted":false,
      "anticipated":false,
      "anticipable":false,
      "lastInvoiceViewedDate":"2021-01-01 12:54:56",
      "lastBankSlipViewedDate":null,
      "postalService":false,
      "creditCard":{
         "creditCardNumber":"8829",
         "creditCardBrand":"MASTERCARD",
         "creditCardToken":"a75a1d98-c52d-4a6b-a413-71e00b193c99"
      },
      "discount":{
         "value":0.00,
         "dueDateLimitDays":0,
         "limitedDate": null,
         "type":"FIXED"
      },
      "fine":{
         "value":0.00,
         "type":"FIXED"
      },
      "interest":{
         "value":0.00,
         "type":"PERCENTAGE"
      },
      "split":[
         {
            "walletId":"48548710-9baa-4ec1-a11f-9010193527c6",
            "fixedValue":20,
            "status":"PENDING",
            "refusalReason": null
         },
         {
            "walletId":"0b763922-aa88-4cbe-a567-e3fe8511fa06",
            "percentualValue":10,
            "status":"PENDING",
            "refusalReason": null
         }
      ],
      "chargeback": {
         "status": "REQUESTED",
         "reason": "PROCESS_ERROR"
      },
      "refunds": null
   }
}
*/

//? will receive webhook from payment gateway when payment is denied
//maybe it could be used to block client in radius aswell

