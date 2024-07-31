use fantoccini::{Client, ClientBuilder, Locator};
use serde::Deserialize;
use tracing::{error, warn};

const LOGIN:&str = "financeiro@smartcomx.com.br";
const PASS:&str = "A7MqZTdEF!M3ctD";

pub async fn checa_voip_down() -> Result<(),anyhow::Error>{
    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
        error!("Erro ao conectar com o chromium webdriver {e}");
        anyhow::anyhow!("Erro a conecar ao webdriver")
    })?;

    login(&client).await.map_err(|e|  {
        error!("Erro ao realizar login no sistema de voip da brdid {e}");
        anyhow::anyhow!("Erro ao logar na brdid")
    })?;

    check_did_registrado(&client).await.map_err(|e| {
        error!("Erro ao checar e reiniciar dids {e}");
        anyhow::anyhow!("Erro ao checar e reiniciar dids")
    })?;

    Ok(())
}


async fn check_did_registrado(client:&Client) -> Result<(),anyhow::Error> {
    client.goto("https://brdid.com.br/br-did/dids/").await.map_err(|e| {
        error!("Failed to navigate to dids page: {}", e);
        anyhow::anyhow!("Failed to navigate to dids page: {}", e)
    })?;

    client.wait().for_element(Locator::Css("tbody tr")).await.map_err(|e| {
        error!("Failed to wait for tbody tr: {}", e);
        anyhow::anyhow!("Failed to wait for tbody tr: {}", e)
    })?;

    let rows = client.find_all(Locator::Css("tbody tr")).await.map_err(|e| {
        error!("Failed to find tbody tr: {}", e);
        anyhow::anyhow!("Failed to find tbody tr: {}", e)
    })?;

    let mut dids_nao_registrados = vec![];

    for row in rows {
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

        //it is expected to not find users with the usuario registrado badge 
        row.find(Locator::Css("span.badge.bg-success i.fa.fa-check")).await.map_err(|e| {
            warn!("Failed to find span.badge.bg-success i.fa.fa-check(usuario registrado): {}", e);
            //Adiciona o dici a lista e nao registrados caso nao se ache o tick de usuario registrado
            //necesario usar uma String para evitar problemas de lifetime
            dids_nao_registrados.push(nome_part.to_string());
            anyhow::anyhow!("Failed to find usuario registrado: {}", e)
        })?;

    }

    let http_client = reqwest::Client::new();
    //it will return json data, we will extract the wan name: InternetGatewayDevice.WANDevice.1.WANConnectionDevice.1.WANPPPConnection.1.Username and the _id
    let routers = http_client.get("http://172.27.27.37:7557/devices").send().await.map_err(|e| {
        error!("Failed to get routers: {}", e);
        anyhow::anyhow!("Failed to get routers: {}", e)
    })?.json::<Vec<Router>>().await.map_err(|e| {
        error!("Failed to parse routers json: {}", e);
        anyhow::anyhow!("Failed to parse routers json: {}", e)
    })?;

    for did in dids_nao_registrados {
        let url = routers.iter().filter(|router| router.wan_name == did).map(|router| {
            warn!("Roteador com DID não registrado: {}", did);
            //formata url com a id do roteador que sera resetado
            format!("http://172.27.27.37:7557/devices/{}/tasks?timeout=3000&connection_request", router._id)
        }).collect::<String>(); 

        //Realiza pedido para reiniciar
        http_client.post(url).json(&serde_json::json!({
            "name": "reboot"
        })).send().await.map_err(|e| {
            error!("Failed to reset router: {}", e);
            anyhow::anyhow!("Failed to reset router: {}", e)
        })?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct Router {
    //BUG this will not work like this i think
    //TODO look at the ways to replicate the json structure for serde
    //dont want to have 8 structs to access the wan name
    #[serde(rename = "InternetGatewayDevice.WANDevice.1.WANConnectionDevice.1.WANPPPConnection.1.Username._value")]
    wan_name:String,
    _id:String,
}

async fn login(client:&Client) -> Result<(), anyhow::Error> {
    //Vai para pagina de login
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
    client.find(Locator::Css("button#wp-submit")).await.map_err(|e| {
        error!("Failed to find wp-submit(login button): {}", e);
        anyhow::anyhow!("Failed to find login button: {}", e)
    })?.click().await.map_err(|e| {
        error!("Failed to click wp-submit(login button): {}", e);
        anyhow::anyhow!("Failed to click on login button: {}", e)
    })?;

    Ok(())
}