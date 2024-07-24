use anyhow::anyhow;
use sqlx::{query, PgPool};
use tracing::error;

pub struct MikrotikNas {
    //This is ip
    pub nasname: String,
    //Name of the mikrotik
    pub shortname: String,
    pub secret: String,
    //I think mikrotik does not have this data
    pub description: String,
}


//TODO will be used for controlling the freeradius db(mysql)
pub async fn insert_mikrotik_to_radius_db(
    pool: &PgPool,
    mikrotik: MikrotikNas,
) -> Result<(), anyhow::Error> {
    query!(
        "INSERT INTO nas(nasname, shortname, type, ports, secret, server, community, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7,$8)",
        mikrotik.nasname,
        mikrotik.shortname,
        "other",
        Option::<i32>::None,
        mikrotik.secret,
        Option::<&str>::None,
        Option::<&str>::None,
        mikrotik.description) 
    .execute(&*pool)
    .await.map_err(|e| -> _ {
        error!("Failed to insert Mikrotik into nas: {:?}", e);
        anyhow!("Failed to insert Mikrotik into nas radius table".to_string())
    })?;

    Ok(())
}


//TODO add users when register cliente is a success(it should call this function after succes,look at how)

//TODO disable users if there is no payment confirmed from webhook after 12 days of the month

//TODO enable users, this will be used to enable users that are disabled(when payment is confirmed from webhook(after day 12 of the month)

