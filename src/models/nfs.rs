use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

#[derive(Serialize, Deserialize,Debug)]
pub struct NfLote {
    pub id: i32,
    pub month: i32,
    pub year: i32,
    pub path: String,
    pub created_at: Option<PrimitiveDateTime>,
}