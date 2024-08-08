use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::PrimitiveDateTime;


#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct PlanoDto {
    pub nome: String,
    pub valor: f32,
    pub velocidade_up: i32,
    pub velocidade_down: i32,
    pub tipo_pagamento: TipoPagamento,
    pub descricao: Option<String>,
    pub contrato_template_id: i32
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Plano {
    pub id: i32,
    pub nome: String,
    pub valor: f32,
    pub velocidade_up: i32,
    pub velocidade_down: i32,
    pub tipo_pagamento: TipoPagamento,
    pub descricao: Option<String>,
    pub contrato_template_id: i32,
    pub created_at : Option<PrimitiveDateTime>,
    pub updated_at : Option<PrimitiveDateTime>
}


#[derive(Deserialize, Serialize, Debug,Clone)]
pub enum TipoPagamento {
    Boleto,
    Pix,
    CartaoCredito,
}

impl FromStr for TipoPagamento {
    type Err = ();

    fn from_str(input: &str) -> Result<TipoPagamento, Self::Err> {
        match input {
            "BOLETO" => Ok(TipoPagamento::Boleto),
            "PIX" => Ok(TipoPagamento::Pix),
            "CREDIT_CARD" => Ok(TipoPagamento::CartaoCredito),
            _ => Err(()),
        }
    }
}

impl From<String> for TipoPagamento {
    fn from(s: String) -> TipoPagamento {
        TipoPagamento::from_str(&s).unwrap_or(TipoPagamento::Boleto) // default to Boleto or handle error appropriately
    }
}

impl ToString for TipoPagamento {
    fn to_string(&self) -> String {
        match self {
            TipoPagamento::Boleto => "BOLETO".to_string(),
            TipoPagamento::Pix => "PIX".to_string(),
            TipoPagamento::CartaoCredito => "CREDIT_CARD".to_string(),
        }
    }
}

// Implementing Into<String> for TipoPagamento
impl Into<String> for TipoPagamento {
    fn into(self) -> String {
        self.to_string()
    }
}