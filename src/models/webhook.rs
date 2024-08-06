use serde::{Deserialize, Serialize};

use super::plano::TipoPagamento;

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomerData {
    pub object: String,
    pub id: String,
    #[serde(rename = "dateCreated")]
    pub date_created: String,
    pub name: String,
    pub email: String,
    #[serde(rename = "cpfCnpj")]
    pub cpf_cnpj: String,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: String,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomerList {
    pub object: String,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
    pub limit: i32,
    pub offset: i32,
    pub data: Vec<CustomerData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientePost{
    pub name: String,
    pub email: String,
    #[serde(rename = "cpfCnpj")]
    pub cpf_cnpj: String,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone:String,
    #[serde(rename = "postalCode")]
    pub postal_code: String,
    #[serde(rename = "addressNumber")]
    pub address_number: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillingType {
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
pub enum Event {
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
pub struct Payment {
    pub id: String,
    #[serde(rename = "dateCreated")]
    pub date_created: String,
    // sera usado para linkar ao cliente
    pub customer: String,
    #[serde(rename = "paymentDate")]
    pub payment_date: Option<String>,
    #[serde(rename = "confirmedDate")]
    pub confirmed_date: Option<String>,
    #[serde(rename = "billingType")]
    pub billing_type: BillingType,
    #[serde(rename = "netValue")]
    pub net_value: f32,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Payload {
    pub id: String,
    pub event: Event,
    #[serde(rename = "dateCreated")]
    pub date_created: String,
    #[serde(rename = "payment")]
    pub payment_data: Payment,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Assinatura {
    pub billing_type: TipoPagamento,
    pub interest: Interest,
    pub fine: Fine,
    pub cycle: String,
    pub value: f32,
    pub customer: String,
    #[serde(rename = "nextDueDate")]
    pub next_due_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fine {
    pub value: f32,
    #[serde(rename = "type")]
    pub fine_type: String,
}  

#[derive(Serialize, Deserialize, Debug)]
pub struct Interest {
    pub value: f32,
}


#[derive(Serialize,Deserialize,Debug)]
pub struct ClienteApi {
    pub id: String,
    #[serde(rename = "cpfCnpj")]
    pub cpf_cnpj: String,
}