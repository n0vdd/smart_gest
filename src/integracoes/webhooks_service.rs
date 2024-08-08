use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::{self}, response::IntoResponse, Extension, Json};
use chrono::{Datelike, Local};
use radius::{bloqueia_cliente_radius, checa_cliente_bloqueado_radius, desbloqueia_cliente};
use sqlx::{query, PgPool};
use time::macros::format_description;
use tracing::{debug, error};

use crate::{clientes::{cliente::find_cliente_by_cpf_cnpj, cliente_model::{ClienteDto, ClienteNf}, plano::find_plano_by_cliente, plano_model::Plano}, financeiro::{nfs_service::gera_nfs, pagamentos::{find_pagamento_confirmado_by_payment_date, save_payment_confirmed_to_db, save_paymente_received_to_db}, pagamentos_model::{PaymentConfirmedDto, PaymentReceivedDto}}, integracoes::webhook_model::{BillingType, ClienteApi, Event}, AppState};

use super::webhook_model::{Assinatura, ClientePost, CustomerList, Fine, Interest, Payload};

///!this is the api key for the sandbox, it should be set on the env for production
const API_KEY: &str = "$aact_YTU5YTE0M2M2N2I4MTliNzk0YTI5N2U5MzdjNWZmNDQ6OjAwMDAwMDAwMDAwMDAwODUzNzI6OiRhYWNoXzAzYTI4MDhmLWI0NmItNDliNC1hNTIwLTRkNWUzZDBjNTQxZg==";

//sandbox
const SANDBOX_USER_URL : &str = "https://sandbox.asaas.com/api/v3/customers/";
const SANDBOX_ASSINATURA_URL : &str = "https://sandbox.asaas.com/api/v3/subscriptions";

//dia 12 do mes os clientes que nao tiverem um pagamento confirmado serao desativados do servidor radius
//feito no scheduler in main

//TODO could test with mocks, i can create some payloads based on examples, and test the handler
//abstract the db calls, so i can test the handler without the db,what do i need to test that not saving to db?
//lida com todos os webhooks do asaas, necessario ser assim para como o webhook funciona
//os eventos que importam sao os flows, pagamento confirmado, pagamento recebido, pagamento estornado
//primeiro recebemos o pagamento confirmado, checamos se e um webhook re-enviado,caso nao, salva na db
//checa se o cliente esta bloqueado, caso esteja, desbloqueia ele
//?talvez eu devesse gerar a nota fiscal de servico aqui, e nao no handler de pagamento recebido, mais provavel de acabar cancelando,nao ideal
//caso de pagamento recebido, checa se o pagamento foi feito por boleto, pix ou cartao de credito(so aceitamos esses tipos de pagamento)
//gera a nota fiscal de servico e envia para o cliente, desbloqueia o cliente caso ele esteja bloqueado
//caso de pagamento estornado, bloqueia o cliente no servidor radius e cancela a nota fiscal(caso a mesma ja tenha sido gerada)
//TODO ainda nao vi como sera feita a parte de cancelar a nota fiscal de servico
pub async fn webhook_handler(
   Extension(pool):Extension<Arc<PgPool>>,State(state):State<AppState>
   ,Json(webhook_data):Json<Payload>) -> impl IntoResponse {
   debug!("Webhook data: {:?}", webhook_data);

   let format = format_description!("[year]-[month]-[day]");
   match webhook_data.event {
      //Essa area apenas lida com db, salva o pagamento confirmado
      //checa se o cliente esta bloqueado, e desbloqueia ele caso o mesmo esteja
      Event::PaymentConfirmed => {
         // Check if the event already exists in payment_confirmed table
         if !check_if_payment_exists(&webhook_data.id, "payment_confirmed", &*pool).await.expect("Erro ao checar se o pagamento ja existe") {
            //TODO should just pass the cliente
            save_payment_confirmed(&pool, &webhook_data, format,&state.http_client).await.expect("Erro ao salvar pagamento confirmado");
         }
         
         let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool,&state.http_client).await.map_err(|e| {
            error!("Failed to fetch client: {:?}", e);
            //TODO maybe i should return a StatusCode
            anyhow::anyhow!("Erro ao buscar cliente no sistema asaas")
         }).expect("Erro ao buscar cliente");

         let cliente = find_cliente_by_cpf_cnpj(&pool, &cliente.cpf_cnpj).await.expect("Erro ao buscar cliente pelo cpf/cnpj");

         if checa_cliente_bloqueado_radius(&cliente.nome).await.map_err(|e| {
            error!("Failed to check if client is blocked: {:?}", e);
            anyhow!("Erro ao checar se o cliente esta bloqueado")
         }).expect("Erro ao checar se o cliente esta bloqueado") {
            let plano = find_plano_by_cliente(&pool, cliente.id).await.map_err(|e| {
               error!("Failed to fetch client plan: {:?}", e);
               anyhow!("Erro ao buscar plano do cliente")
            }).expect("Erro ao buscar plano do cliente");

            desbloqueia_cliente(&cliente.nome, plano.nome).await.map_err(|e| {
               error!("Failed to unblock client: {:?}", e);
               anyhow!("Erro ao desbloquear cliente")
            }).expect("Erro ao desbloquear cliente");
         }
         http::StatusCode::OK
      }, 

      //Salva o pagamento recebido, gera a nota fiscal de servico e manda por email para o cliente
      //desbloqueia o cliente caso ele esteja bloqueado
      Event::PaymentReceived => {
         match webhook_data.payment_data.billing_type {
            BillingType::Boleto | BillingType::Pix | BillingType::CreditCard => {
               debug!("Gerando nota fiscal de servico para cliente: {:?}", webhook_data.payment_data.customer);

               let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool,&state.http_client).await.map_err(|e| {
                  error!("Failed to fetch client: {:?}", e);
                  e
               }).expect("Erro ao buscar cliente");

               let cliente = find_cliente_by_cpf_cnpj(&pool, &cliente.cpf_cnpj).await.expect("Erro ao buscar cliente pelo cpf/cnpj");

               let cliente_nf = ClienteNf {
                  nome: cliente.nome.clone(),
                  email: cliente.email.clone(),
                  cpf_cnpj: cliente.cpf_cnpj.clone(),
                  gera_nf: cliente.gera_nf,
                  rua: cliente.rua.clone(),
                  numero: cliente.numero.clone(),
                  //bairro: cliente.bairro.clone(),
                  //cidade: cliente.cidade.clone(),
                  //estado: cliente.estado.clone(),
                  complemento: cliente.complemento.clone(),
                  cep: cliente.cep.clone(),
               };

               if checa_cliente_bloqueado_radius(&cliente.nome).await.map_err(|e| {
                  error!("Failed to check if client is blocked: {:?}", e);
                  anyhow!("Erro ao checar se o cliente esta bloqueado")
               }).expect("Erro ao checar se o cliente esta bloqueado") {
                  let plano = find_plano_by_cliente(&pool, cliente.id).await.map_err(|e| {
                     error!("Failed to fetch client plan: {:?}", e);
                     anyhow!("Erro ao buscar plano do cliente")
                  }).expect("Erro ao buscar plano do cliente");

                  desbloqueia_cliente(&cliente.nome, plano.nome).await.map_err(|e| {
                     error!("Failed to unblock client: {:?}", e);
                     anyhow!("Erro ao desbloquear cliente")
                  }).expect("Erro ao desbloquear cliente");
               }
               
               // Check if the event already exists in payment_received table
               if !check_if_payment_exists(&webhook_data.id, "payment_received", &*pool).await.expect("Erro ao checar se o pagamento ja existe") {
                  let id = save_payment_received(&pool, &webhook_data, format).await.expect("Erro ao salvar pagamento recebido");
                  gera_nfs(&pool,&cliente_nf,webhook_data.payment_data.net_value,state.mailer,id).await.context("Erro ao gerar nota fiscal de servico")
                     .expect("Erro ao gerar nota fiscal");
               }
            },
            //Flow com algum tipo de pagamento sem ser boleto/pix ou cartao
            //nao criamos esse tipo de assinatura para os clientes
            _ => {
               //TODO mandar algum aviso e logar, nao deveria nem chegar nesse flow
               //so vendemoos boleto, cartao de credito e pix
               error!("Tipo de pagamento nao suportado: {:?}", webhook_data.payment_data.billing_type);

               //TODO enviar notificao pelo telegram
//               reqwest::Client::new().get(url)
            }
         }

         //gera nota fiscal e envia para o cliente

         http::StatusCode::OK
      },

      Event::PaymentRefunded => {
         //TODO cancelar nota fiscal de servico
         if Local::now().day() >= 12 {
            let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool,&state.http_client).await.map_err(|e| {
               error!("Failed to fetch client: {:?}", e);
               anyhow!("Erro ao buscar cliente no sistema asaas")
            }).expect("Erro ao buscar cliente");

            let cliente = find_cliente_by_cpf_cnpj(&pool, &cliente.cpf_cnpj).await.expect("Erro ao buscar cliente pelo cpf/cnpj");

            bloqueia_cliente_radius(&cliente.login).await.map_err(|e| {
               error!("Failed to block client: {:?}", e);
               anyhow!("Erro ao bloquear cliente no servidor radius")
            }).expect("Erro ao bloquear cliente");

         }
         http::StatusCode::OK
      }, 

      Event::PaymentRefundInProgress => {
         //TODO cancelar nota fiscal de servico
         if Local::now().day() >= 12 {
            let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool,&state.http_client).await.map_err(|e| {
               error!("Failed to fetch client: {:?}", e);
               anyhow!("Erro ao buscar cliente no sistema asaas")
            }).expect("Erro ao buscar cliente");

            let cliente = find_cliente_by_cpf_cnpj(&pool, &cliente.cpf_cnpj).await.expect("Erro ao buscar cliente pelo cpf/cnpj");

            bloqueia_cliente_radius(&cliente.login).await.map_err(|e| {
               error!("Failed to block client: {:?}", e);
               anyhow!("Erro ao bloquear cliente no servidor radius")
            }).expect("Erro ao bloquear cliente");
         }
         http::StatusCode::OK
      },

      _ => {
         http::StatusCode::OK
      }
   }
}


//TODO o codigo que chama esse deve mapear  o erro para um StatusCode
//Cria o cliente e a assinatura
pub async fn add_cliente_to_asaas(cliente:&ClienteDto,plano:&Plano,client:&reqwest::Client) -> Result<(), anyhow::Error> {
   //checa a flag setada ao cadastrar o cliente
   //true por padrao
   if cliente.add_to_asaas == false {
      return Ok(());
   }

   //Pega uma  lista com todos os clientes
   client.get(SANDBOX_USER_URL).header("access_token",API_KEY)
      .header("accept", "application/json")
      .header("user-agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
      .send().await
      .map_err(|e | {
         error!("Failed to fetch clients: {:?}", e);
         anyhow::anyhow!("Erro ao buscar clientes do sistema asaas")
      })?
      .json::<CustomerList>().await
      .map_err(|e| {
         error!("Failed to parse clients: {:?}", e);
         anyhow::anyhow!("Erro ao realizar parse dos clientes")
      })?
      //Checa se o nome do cliente ja esta no sistema do asaas 
      .data.iter().find(|cl| cl.name == cliente.nome);

   let post_cliente = ClientePost {
      name: cliente.nome.clone(),
      email: cliente.email.clone(),
      cpf_cnpj: cliente.cpf_cnpj.clone(),
      mobile_phone: cliente.telefone.clone(),
      postal_code: cliente.endereco.cep.clone(),
      address_number: cliente.endereco.numero.clone().unwrap()
   };


   //Envia o cliente caso ele nao exista
   let asaas_cliente = client.post(SANDBOX_USER_URL).header("access_token",API_KEY)
      .header("accept", "application/json")
      .header("content-type", "application/json")
      .header("user-agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
      .json(&post_cliente).send().await.map_err(|e| {
         error!("Failed to post client: {:?}", e);
         anyhow::anyhow!("Erro ao enviar cliente para sistema do asaas")
      })?.json::<ClienteApi>().await.map_err(|e| {
         error!("Failed to parse client: {:?}", e);
         anyhow::anyhow!("Erro ao realizar parse do cliente")
      })?;

   add_assinutara_cliente_asaas(&asaas_cliente,plano,client).await.map_err(|e| {
      error!("Failed to add subscription: {:?}", e);
      anyhow::anyhow!("Erro ao adicionar assinatura ao cliente")
   })?;

   Ok(())
}

async fn add_assinutara_cliente_asaas(cliente_asaas:&ClienteApi,plano:&Plano,client:&reqwest::Client) -> Result<(),anyhow::Error>{
   let month = Local::now().month();
   let year = Local::now().year();
   //Smartcom comeca a cobrar 3 meses depois, sempre no dia 5
   let next_due_date= format!("{}-{}-5",year,month+3);


   let client = client.post(SANDBOX_ASSINATURA_URL)
   .header("accept", "application/json")
   .header("content-type", "application/json")
   .header("user-agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36");

   let assinatura = Assinatura {
      billing_type: plano.tipo_pagamento.clone(),
      //TODO deveria ser definido pelo plano
      interest: Interest {
         value: 1.0
      },
      //TODO deveria ser definido pelo plano
      fine: Fine {
         value: 2.0,
         fine_type: "PERCENTAGE".to_string()
      },
      //Pagamento de internet costuma ser mensal
      //TODO deveria ser definido ao criar o plano
      cycle: "MONTHLY".to_string(),
      value: plano.valor,
      customer: cliente_asaas.id.clone(),
      next_due_date
   };

   //save assinatura to the asaas system
   client.json(&assinatura).send().await.map_err(|e| {
      error!("Failed to post subscription: {:?}", e);
      anyhow::anyhow!("Erro ao enviar assinatura para o sistema do asaas")
   })?;

   Ok(())
} 

//TODO pass this to the pagamento code
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


//TODO usar api key no ambiente
//Esse codigo so pega dado do asaas e comparo com a db, nao ha oque testar
//TODO acha o cliente no sistema pelo id do asaas(poderia salvar o id do asaas na tabela do cliente(mais simples))
//so precisaria pegar a id do cliente ao criar ele e colocar na db 
//usar o reqwest cliente a partir do estado da aplicacao
async fn find_api_cliente(id:&str,pool: &PgPool,client: &reqwest::Client) -> Result<ClienteApi, anyhow::Error> {
   //TODO send a request to this url: https://sandbox.asaas.com/api/v3/customers/{id}
   //producao: https://www.asaas.com/api/v3/customers/{id}

   let url = format!("{}{}",SANDBOX_USER_URL,id);
   //get the cpfCnpj from the response and use it to find the cliente in the db
   let client = client.get(url)
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
   
   Ok(client)
}

///!This code is called only after a check if the event if exists on the db
async fn save_payment_received(pool:&PgPool,webhook_data: &Payload,format: &[time::format_description::BorrowedFormatItem<'_>]) -> Result<i32,anyhow::Error>{
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
      let payment_confirmed = find_pagamento_confirmado_by_payment_date(&pool, data)
         .await.expect("Erro ao buscar pagamento confirmado pela data"); 

      //Gets the id of the payment_confirmed if it exists
      if let Some(confirmed_id) = payment_confirmed {
         if let Some(data) = &webhook_data.payment_data.payment_date {
            // Format time for PostgreSQL
            let data = time::Date::parse(data, format).map_err(|e| {
               error!("Failed to parse date: {:?}", e);
               anyhow::anyhow!("Erro ao fazer parse de uma string em uma data")
            })?;

            let payment = PaymentReceivedDto {
               event_id: webhook_data.id.clone(),
               payment_confirmed: confirmed_id.id,
               payment_date: data
            };

            match save_paymente_received_to_db(&pool, &payment).await {
               Ok(id) => Ok(id),

               Err(e) => {
                  error!("Failed to save payment: {:?}", e);
                  Err(anyhow::anyhow!("Erro ao salvar pagamento"))
               }
            }

         } else {
            anyhow::bail!("Data de pagamento nao encontrada")
         }
      } else {
         anyhow::bail!("Pagamento confirmado nao encontrado")
      }
   } else {
      anyhow::bail!("Data de pagamento confirmado nao encontrada")
   }
}

///!This code is called only after a check if the event if exists on the db
//TODO this could get the cliente, there is a call to find_api_cliente after this code, so can use the cliente data directly,and dont call it again
async fn save_payment_confirmed(pool:&PgPool,webhook_data: &Payload,format: &[time::format_description::BorrowedFormatItem<'_>],client:&reqwest::Client) -> Result<(),anyhow::Error>{
   debug!("Salvando pagamento confirmado: {:?}", webhook_data);
   //TODO maybe should check the event_id before saving it?

   //find the cliente from the asaas api
   let cliente = find_api_cliente(&webhook_data.payment_data.customer, &pool,client).await.map_err(|e| {
      error!("Failed to fetch client: {:?}", e);
      anyhow::anyhow!("Erro ao buscar cliente no sistema asaas")
   })?;

   let cliente = find_cliente_by_cpf_cnpj(&pool, &cliente.cpf_cnpj).await.expect("Erro ao buscar cliente pelo cpf/cnpj");

   // Format time for PostgreSQL
   if let Some(data) = &webhook_data.payment_data.payment_date {
      let data = sqlx::types::time::Date::parse(data, format).map_err(|e| {
         error!("Failed to parse date: {:?}", e);
         anyhow::anyhow!("Erro ao realizar parse de uma string em uma data")
      })?;

      let payment = PaymentConfirmedDto {
         event_id: webhook_data.id.clone(),
         cliente_id: cliente.id,
         payment_date: data
      };

      save_payment_confirmed_to_db(&pool, &payment).await.expect("Erro ao salvar pagamento confirmado");
   }
   Ok(())
}

//? will receive webhook from payment gateway when payment is denied
//maybe it could be used to block client in radius aswell

