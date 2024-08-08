use time::{Date, PrimitiveDateTime};

pub struct PaymentConfirmed {
    pub id: i32,
    pub event_id: String,
    pub cliente_id: i32,
    pub payment_date: Date,
    pub created_at: Option<PrimitiveDateTime>,
}

pub struct PaymentConfirmedDto {
    pub event_id: String,
    pub cliente_id: i32,
    pub payment_date: Date,
}

pub struct PaymentReceivedDto {
    pub event_id: String,
    pub payment_confirmed: i32,
    pub payment_date: Date,
}