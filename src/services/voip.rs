use anyhow::Context;
use fantoccini::{Client, ClientBuilder, Locator};
use serde::Deserialize;
use tracing::{debug, error, warn};

const LOGIN:&str = "financeiro@smartcomx.com.br";
const PASS:&str = "A7MqZTdEF!M3ctD";

//TODO this could all be done with reqwest i think
pub async fn checa_voip_down() -> Result<(),anyhow::Error>{
    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
        error!("Erro ao conectar com o chromium webdriver {e}");
        anyhow::anyhow!("Erro a conecar ao webdriver")
    })?;
    //let client = reqwest::Client::new();
    debug!("Iniciando login");
    login(&client).await.map_err(|e|  {
        error!("Erro ao realizar login no sistema de voip da brdid {e}");
        anyhow::anyhow!("Erro ao logar na brdid")
    })?;

    debug!("Checando dids registrados");
    check_did_registrado(&client).await.map_err(|e| {
        error!("Erro ao checar e reiniciar dids {e}");
        anyhow::anyhow!("Erro ao checar e reiniciar dids")
    })?;

    Ok(())
}


//TODO should check if there is more pages
async fn check_did_registrado(client:&Client) -> Result<(),anyhow::Error> {
    client.goto("https://brdid.com.br/br-did/dids/").await.map_err(|e| {
        error!("Failed to navigate to dids page: {}", e);
        anyhow::anyhow!("Failed to navigate to dids page: {}", e)
    })?;

    debug!("Esperando a listagem dos dids carregar");
    client.wait().for_element(Locator::Css("tbody tr")).await.map_err(|e| {
        error!("Failed to wait for tbody tr: {}", e);
        anyhow::anyhow!("Failed to wait for tbody tr: {}", e)
    })?;

    debug!("pegando as colunas da listagem");
    let rows = client.find_all(Locator::Css("tbody tr")).await.map_err(|e| {
        error!("Failed to find tbody tr: {}", e);
        anyhow::anyhow!("Failed to find tbody tr: {}", e)
    })?;

    let mut dids_nao_registrados = vec![];

    debug!("iterando sobre as linhas da listagem");
    for row in rows {
        debug!("selecionando o nome do cliente");
        let nome = row.find(Locator::Css("td:nth-child(2)")).await.map_err(|e| {
            error!("Failed to find td:nth-child(2)(nome do cliente): {}", e);
            anyhow::anyhow!("Failed to find nome do cliente: {}", e)
        })?.text().await.map_err(|e| {
            error!("Failed to get text from td:nth-child(2): {}", e);
            anyhow::anyhow!("Failed to get text from td:nth-child(2): {}", e)
        })?;

        
        if !nome.contains("-") {
            warn!("Nome do cliente não contem '-': {}", nome);
            continue;
        }

        let nome_part = nome.split('-').nth(0).expect("erro ao pegar nome do cliente").trim();
        debug!("Nome do cliente: {}", nome_part);

        //it is expected to not find users with the usuario registrado badge 
        debug!("Procurando pelo tick de usuario registrado");
        //TODO this should not stop when it errors
        row.find(Locator::Css("span.badge.bg-success i.fa.fa-check")).await.map_err(|e| {
            warn!("Failed to find span.badge.bg-success i.fa.fa-check(usuario registrado): {}", e);
            //Adiciona o dici a lista e nao registrados caso nao se ache o tick de usuario registrado
            //necesario usar uma String para evitar problemas de lifetime
            dids_nao_registrados.push(nome_part.to_string());
            anyhow::anyhow!("Failed to find usuario registrado: {}", e)
        }).ok();

    }

    debug!("Dids nao registrados: {:?}", dids_nao_registrados);

    debug!("Fetching routers...");
    let http_client = reqwest::Client::new();
    let routers_response = http_client.get("http://172.27.27.37:7557/devices?projection=InternetGatewayDevice.WANDevice.1.WANConnectionDevice.1.WANPPPConnection.1.Username,_id")
        .send().await.context("Failed to get routers")?
        .text().await.context("Failed to get routers text")?;
    debug!("Routers response: {}", routers_response);

    let routers: serde_json::Value = serde_json::from_str(&routers_response).context("Failed to parse routers JSON")?;
    let routers = routers.as_array().context("Expected an array of routers")?;

    let mut unregistered_routers = vec![];
    for router in routers {
        if let Some(router_id) = router.get("_id").and_then(serde_json::Value::as_str) {
            if let Some(username) = router.pointer("/InternetGatewayDevice/WANDevice/1/WANConnectionDevice/1/WANPPPConnection/1/Username/_value").and_then(serde_json::Value::as_str) {
                if dids_nao_registrados.contains(&username.to_string()) {
                    unregistered_routers.push((router_id, username));
                }
            }
        }
    }

    debug!("Rebooting unregistered routers...");
    for (router_id, username) in unregistered_routers {
        warn!("Rebooting router with DID not registered: {} ({})", router_id, username);
        let reset_url = format!("http://172.27.27.37:7557/devices/{}/tasks?timeout=3000&connection_request", router_id);
        http_client.post(&reset_url)
            .json(&serde_json::json!({ "name": "reboot" }))
            .send().await.map_err(|e| {
                error!("Failed to reset router: {}", e);
                anyhow::anyhow!("Failed to reset router: {}", e)
            })?;
    }

    Ok(())
}

#[derive(Deserialize,Debug)]
struct Router {
    //BUG this will not work like this i think
    //TODO look at the ways to replicate the json structure for serde
    //dont want to have 8 structs to access the wan name
    wan_name:String,
    _id:String,
}

//TODO this could be done with reqwest
//post the login data
async fn login(client:&Client) -> Result<(), anyhow::Error> {
    /*realiza login
    let pagina_principal = client.post("https://brdid.com.br/br-did/wp-login.php").form(&[("user_login", LOGIN), ("user_pass", PASS)]).send().await.map_err(|e| {
        error!("Failed to login: {}", e);
        anyhow::anyhow!("Failed to login: {}", e)
    })?;
    */

    client.goto("https://brdid.com.br/br-did/").await.map_err(|e| {
        error!("Failed to navigate: {}", e);
        anyhow::anyhow!("Failed to navigate: {}", e)
    })?;

    //esperar a pagina carregar
    client.wait().for_element(Locator::Css("input#user_login")).await.map_err(|e| {
        error!("Failed to wait for user_login: {}", e);
        anyhow::anyhow!("Failed to wait for user_login: {}", e)
    })?;

    //Preencher o campo de login
    client.find(Locator::Css("input#user_login")).await.map_err(|e| {
        error!("Failed to find user_login: {}", e);
        anyhow::anyhow!("Failed to find user_login: {}", e)
    })?
    .send_keys(LOGIN).await.map_err(|e| {
        error!("Failed to send_keys to user_login: {}", e);
        anyhow::anyhow!("Failed to send_keys to user_login: {}", e)
    })?;

    //preecher o campo de senha
    client.find(Locator::Css("input#user_pass")).await.map_err(|e| {
        error!("Failed to find user_pass: {}", e);
        anyhow::anyhow!("Failed to find user_pass: {}", e)
    })?
    .send_keys(PASS).await.map_err(|e| {
        error!("Failed to send_keys to user_pass: {}", e);
        anyhow::anyhow!("Failed to send_keys to user_pass: {}", e)
    })?;

    //logar
    client.find(Locator::Id("wp-submit")).await.map_err(|e| {
        error!("Failed to find wp-submit(login button): {}", e);
        anyhow::anyhow!("Failed to find login button: {}", e)
    })?.click().await.map_err(|e| {
        error!("Failed to click wp-submit(login button): {}", e);
        anyhow::anyhow!("Failed to click on login button: {}", e)
    })?;

    Ok(())
}