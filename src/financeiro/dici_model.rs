use serde::{Deserialize, Serialize};
use time::{Date, PrimitiveDateTime};

#[derive(Deserialize)]
pub struct GenerateDiciForm {
    pub reference_date: String,
}

#[derive(Deserialize,Serialize,Debug)]
pub struct Dici {
    pub id: i32,
    pub path: String,
    pub reference_date: Date,
    pub created_at: Option<PrimitiveDateTime> 
}

pub struct DiciDto {
    pub path: String,
    pub reference_date: Date,
}