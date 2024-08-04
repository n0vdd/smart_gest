use std::sync::Arc;

use axum::Extension;
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::models::config::{NfConfig, Provedor};

//TODO this will show the form to create/edit the provedor of the system
pub async fn show_provedor_form(Extension(pool):Extension<Arc<PgPool>>,Form(provedor):Form<Provedor>) {
    if provedor.id == 0 {
        //TODO show the form to create a new provedor
    } else {
        //TODO show the form to edit the provedor
        
    }
}

pub async fn show_nf_config(Extension(pool):Extension<Arc<PgPool>>,Form(form):Form<NfConfig>) {
    //TODO show the form to configure the NF
}

