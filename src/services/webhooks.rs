use std::sync::Arc;

use axum::{http::{self, status::InvalidStatusCode}, response::IntoResponse, Extension, Json};
use chrono::{Date, Datelike, Local};
use serde::{Deserialize, Serialize};
use sqlx::{pool, query, query_as, PgPool};
use time::macros::format_description;
use tracing::{debug, error};

use crate::handlers::clients::Cliente;

//TODO implemente a payment received struct for the webhook

#[derive(Serialize,Deserialize,Debug)]
struct Payment{
   id: String,
   event: String,//will always be PAYMENT_RECEIVED,this need to be checked
   #[serde(rename = "dateCreated")]
   date_created: String,
   //sera usado para linkar ao cliente
   customer: String,
   #[serde(rename = "paymentDate")]
   payment_date: Option<String>,
   #[serde(rename = "confirmedDate")]
   confirmed_date: Option<String>,


}

#[derive(Serialize,Deserialize,Debug)]
pub struct Payload {
      id: String,
      event: String,
      #[serde(rename = "dateCreated")]
      date_created: String,
      payment_data: Payment,
}

//todo dia 12 do mes os clientes que nao tiverem um pagamento confirmado serao desativados do servidor radius
//TODO gerar nota fiscal de servico apos receber pagamento
//TODO radius deveria checar todo dia 12 os clientes que nao tem um pagamente confirmado 
pub async fn webhook_handler(Extension(pool):Extension<Arc<PgPool>>,Json(webhook_data):Json<Payload>) -> impl IntoResponse {
      let format = format_description!("[year]-[month]-[day]");
      //TODO first check if the payment is already in the database
      match webhook_data.event.as_str() {
         "PAYMENT_CONFIRMED" => {

            let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool).await.map_err(|e| {
               error!("Failed to fetch client: {:?}", e);
               e
            }).expect("Erro ao buscar cliente");

            //format time for postrgresql
            let format = format_description!("[year]-[month]-[day]");

            if let Some(data) = &webhook_data.payment_data.payment_date {

               let data = sqlx::types::time::Date::parse(data, format).map_err(|e| {
                  error!("Failed to parse date: {:?}", e);
                  e
               }).expect("Erro ao tranformar string em uma data");

               query!("INSERT INTO payment_confirmed (event_id,cliente_id,payment_date) VALUES ($1,$2,$3)"
               ,webhook_data.id,cliente.id,data).execute(&*pool).await.map_err(|e| {
                  error!("Failed to save payment: {:?}", e);
                  e
               }).expect("Erro ao salvar pagamento");
            }
            http::StatusCode::OK
         }, 
         //TODO caso cartao de credito e possivel a compra ser estornada(preciso lidar com esse caso)
         //caso seja depois do dia 12, bloquear o cliente tambem
         "PAYMENT_RECEIVED" => {
            if let Some(confirmed_data) = &webhook_data.payment_data.confirmed_date {
               let data = sqlx::types::time::Date::parse(confirmed_data, format).map_err(|e| {
                  error!("Failed to parse date: {:?}", e);
                  e
               }).expect("Erro ao tranformar string em uma data");

               let payment_confirmed = query!(
                  "SELECT id FROM payment_confirmed WHERE payment_date  = $1",
                  data 
               ).fetch_optional(&*pool).await.map_err(|e| {
                  error!("Failed to fetch payment: {:?}", e);
                  e
               }).expect("Erro ao buscar pagamento");

               if let Some(confirmed_id) = &payment_confirmed {

                  if let Some(data) = &webhook_data.payment_data.payment_date {

                     let data = sqlx::types::time::Date::parse(data, format).map_err(|e| {
                        error!("Failed to parse date: {:?}", e);
                        e
                     }).expect("Erro ao tranformar string em uma data");

                     query!("INSERT INTO payment_received (event_id,payment_confirmed,payment_date) 
                        VALUES ($1,$2,$3)",webhook_data.id,
                        confirmed_id.id,data)
                        .execute(&*pool)
                        .await.map_err(|e| {
                           error!("erro ao salvar pagamento {:?}",e);
                           e
                        }).expect("Erro ao salvar pagamento");

                        //TODO get the data necessary for nfs generation(relation is in plano trough cliente)


                  }
               }
            }
            //TODO send an email to the client with the nota fiscal
            //TODO send a 200 status code to the payment gateway
            axum::http::StatusCode::OK
         },
         "PAYMENT_REFUNDED" => {
            //TODO block client if this is after day 12
            if Local::now().day() > 12 {
               
            }
            http::StatusCode::OK
         },
         _ => {
            http::StatusCode::OK
            //TODO send a 200 status code to the payment gateway
         }
      }
   }
   //Will save to the db with its id and the client it references to and the payed date
   //TODO will have to create a nota fiscal de servico for the client
   //save it to fs and reference it on the db aswell
   //send an email with the nota_fiscal



//check if the payment is already in the db
//return true if it is, false if it is not
async fn check_if_payment_exists(id: &str,pool:&PgPool) -> bool {
      let event_id = query!(
         "SELECT event_id FROM payment_received WHERE event_id = $1",
         id
      ).fetch_optional(pool).await.map_err(|e| {
         error!("Failed to fetch payment: {:?}", e);
         e
      }).expect("Erro ao buscar pagamento");
      if event_id.is_none() {
         false
      } else {
         true
      }
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

