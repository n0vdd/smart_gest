use std::sync::Arc;

use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::{pool, PgPool};

//TODO implemente a payment received struct for the webhook

#[derive(Serialize,Deserialize,Debug)]
struct PaymentReceived {
   id: String,
   event: String,//will always be PAYMENT_RECEIVED,this need to be checked
   #[serde(rename = "dateCreated")]
   date_created: String,
   //sera usado para linkar ao cliente
   customer: String,
   #[serde(rename = "paymentDate")]
   payment_date: String
}

#[derive(Serialize,Deserialize,Debug)]
struct Payload {
      id: String,
      event: String,
      #[serde(rename = "dateCreated")]
      date_created: String,

}

//TODO will receive webhook from payment gateway when payment is confirmed
//todo dia 12 do mes os clientes que nao tiverem um pagamento confirmado serao desativados do servidor radius
//TODO gerar nota fiscal de servico apos receber pagamento
async fn webhook_handler(Json(webhook_data):Json<Payload>,Extension(pool):Extension<Arc<PgPool>>) {
    //TODO first check if the payment is already in the database
    if check_if_payment_exists(webhook_data.id,&*pool).await {
        //TODO send a 200 status code to the payment gateway
        return;
    } else {
      match webhook_data.event.as_str() {
         "PAYMENT_RECEIVED" => {
               //TODO save the payment to the db
               //TODO send an email to the client with the nota fiscal
               //TODO send a 200 status code to the payment gateway
         },
         "PAYMENT_DECLINED" => {
         }
         _ => {
               //TODO send a 200 status code to the payment gateway
               //? should not happen, look at the docs at what to do with an unknown event
         }
      }
   }
   //Will save to the db with its id and the client it references to and the payed date
   //! always check if the id is already in the db before dealing with the webhook_data
   //! need to return a 200 status code to the payment gateway before processing it

   //TODO will have to create a nota fiscal de servico for the client
   //save it to fs and reference it on the db aswell
   //send an email with the nota_fiscal
}

async fn check_if_payment_exists(id: String,pool:&PgPool) -> bool {
    //TODO check if the payment is already in the db
    //TODO return true if it is, false if it is not

    todo!()
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

