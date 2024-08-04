use time::PrimitiveDateTime;

pub struct Provedor {
    pub id: i32,
    pub nome: String,
    pub cnpj: String,
    pub cep: String,
    pub rua: String,
    pub numero: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub estado: String,
    pub complemento: Option<String>,
    pub telefone: String,
    pub email: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>,
}

//Just set the email that will receive the nf lote
pub struct NfConfig {
    pub id: i32,
    pub contabilidade_email: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>
}

pub struct EmailConfig {
    pub id: i32,
    pub login: String,
    pub password: String,
    //TODO use ipv4
    smtp_server: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub updated_at: Option<PrimitiveDateTime>
}