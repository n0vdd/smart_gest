//TODO automatizar criacao de nota fiscal de servico
//pelo site da prefeitura de nova lima
//talvez tenha como enviar por xml(mais dificil)
//? devo precisar de alguma crate para xml(tenho alguns docs sobre)
//talvez precisa auatomatizar como se fosse uma pessoa(eles nao devem verificar bots, so nao posso derrubar o site)

//enviar um email com a nota gerada apos pagamento


const CNPJ: &str = "48530335000148";
const PASSWORD: &str = "oU6jlxL7RpUY7JB3TAqD";
const ID_MUNICIPIO: &str = "17428";
const ID_SERVICO: &str= "103";
const ID_CNAE: &str = "117019000";
const DESCRICAO: &str = "Serviço de internet";



use std::time::{Duration, SystemTime};

use anyhow::Context;
use chrono::{Datelike,Local};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use serde_json::json;
use sqlx::{query, PgPool};
use tokio::{fs::{read_dir, rename}, time::sleep};
use std::path::PathBuf;

use fantoccini::{wd::Capabilities, Client, ClientBuilder, Locator};
use tracing::{debug, error, info, warn};

use crate::{models::client::ClienteNf, services::email::send_nf};

use super::email::send_nf_lote;

//TODO download the nf emitida e nao enviada
//irei testar tanto o envio de email para o cliente
// quanto o processo de fazer download do arquivo pelo botao imprimir
pub async fn download_nf_nao_enviada(pool:&PgPool,mailer: AsyncSmtpTransport<Tokio1Executor>) -> Result<(),anyhow::Error> {
    let caps = setup_gera_nf_chromedriver("teste").await;

    let client = ClientBuilder::native().capabilities(caps)
    .connect("http://localhost:9515").await.context("Erro ao conectar ao WebDriver")?;

    //TODO login to the system
    login(&client).await.context("Erro ao logar no sistema de nota fiscal")?;

    //wait for the login to complete
    //BUG if i do the next step too fast it will fail,so its better to wait until the page after login loads
    client.wait().for_element(Locator::Id("span_vVSAIDA")).await.context("Erro ao esperar pelo elemento de texto")?;

    client.find(Locator::Id("span_vVSAIDA")).await.context("Erro ao encontrar o elemento de texto")?
    .click()
    .await.context("Erro ao clicar no elemento notas emitidas")?;

    client.wait().for_element(Locator::Id("vVISUALIZAR_0001")).await.context("Erro ao esperar pelo elemento para visualizar nota emitida")?;

    client.find(Locator::Id("vVISUALIZAR_0001")).await.context("Erro ao encontrar o elemento para visualizar nota emitida")?
    .click()
    .await.context("Erro ao clicar no elemento para visualizar nota emitida")?;

    let nf = salvar_nota_fiscal(&client, "teste")
        .await.context("Erro ao salvar nota fiscal")?;    


    /* 
    //maybe this will already save the pdf, if not will need to click on the save button
    client.wait().for_element(Locator::Id("content")).await.context("failed to find salvar button")?;

    client.find(Locator::Id("content")).await.context("failed to find salvar button")?
    .click()
    .await.context("failed to click salvar button")?;
    */
    /* 
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
    let path = "nota_fiscal/teste";
    let mut dir = read_dir(&path).await.context("failed to read dir")?;
    // Variables to track the last modified file
    let mut last_modified_file: Option<(PathBuf, SystemTime)> = None;

    while let Some(entry) = dir.next_entry().await.context("Failed to get next entry")? {
        let metadata = entry.metadata().await.context("Failed to get metadata")?;
        let modified_time = metadata.modified().context("Failed to get modified time")?;

        match last_modified_file {
            Some((_, last_modified_time)) => {
                if modified_time > last_modified_time {
                    last_modified_file = Some((entry.path().clone(), modified_time));
                }
            }
            None => {
                last_modified_file = Some((entry.path().clone(), modified_time));
            }
        }

        // Rename the last modified file to include the current date
        if let Some((last_file_path, _)) = last_modified_file.clone() {
            let current_date = Local::now().format("%d-%m-%Y").to_string();
            let new_file_name = format!("{}/{}.pdf", path, current_date);
            rename(&last_file_path, &new_file_name).await.context("Failed to rename file")?;
        } else {
            warn!("No files found in the directory.");
        }
    }
*/
    Ok(())
}


//TODO cancelar nota fiscal, necessario checar se a nota fiscal existe
//cancela a mesma, tera que ser passada o cliete relevante e a data imagino
pub async fn cancela_nfs() -> Result<(),anyhow::Error> {
 

    //TODO navegar para essa url: https://e-nfs.com.br/e-nfs_novalima/servlet/hwconsultaprocessocontrib

    //TODO clicar em button2

    //TODO adicionar o motivo(dados errados ou coisa do tipo)
    //servico cancelado sla

    //achar id da nota fiscal, posso pesquisar no sistema de arquivo pela data ou coisa do tipo
    //colocar id no campo vNFSNUMERO
    //clicar em BTNINCLUIRNOTA

    //confirma clicando em BUTTON2
    
    //TODO enviar e receber a nota fiscal cancelada
    //salvar a mesma para o sistema de arquivos
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
    Ok(())
}

fn setup_export_chromedriver() -> Capabilities {
    // Define the Chrome options with download preferences
    //BUG download dir e um caminho completo, quando mudar de servidor tenho que alterar aqui
    let download_dir = "/home/user/code/smart_gest/nota_fiscal/export_lotes/"; 

    let mut prefs = serde_json::Map::new();
    prefs.insert("download.default_directory".to_string(), serde_json::Value::String(download_dir.to_string()));

    let mut chrome_options = serde_json::Map::new();
    //prefs.insert("directory_upgrade".to_string(), serde_json::Value::Bool(true));
    chrome_options.insert("prefs".to_string(), serde_json::Value::Object(prefs));

    let mut caps = Capabilities::new();
    caps.insert("goog:chromeOptions".to_string(), serde_json::Value::Object(chrome_options));

    caps
}

pub async fn exporta_nfs(pool: &PgPool,mailer:&AsyncSmtpTransport<Tokio1Executor> ) -> Result<(),anyhow::Error> {
    let caps = setup_export_chromedriver();

    let client = ClientBuilder::native()
    .capabilities(caps)
    .connect("http://localhost:9515").await
    .context("Erro ao conectar ao WebDriver")?;

    login(&client).await?;

    //wait for the login to complete
    //BUG if i do the next step too fast it will fail,so its better to wait until the page after login loads
    client.wait().for_element(Locator::Id("TEXTBLOCK15")).await.context("Erro ao esperar pelo elemento de texto")?;

    client.goto("https://e-nfs.com.br/e-nfs_novalima/servlet/hwmcontabilidade").await.context("Erro ao navegar para a pagina de exportar nota fiscal")?;

    //wait for the page to load
    client.wait().for_element(Locator::Id("vDATAINICIO")).await.context("failed to find start date input element")?;

    // Open the first calendar
    client.find(Locator::Id("vDATAINICIO_dp_trigger")).await.context("failed to find start date trigger element")?
    .click()
    .await.context("failed to click start date trigger")?;

    //wait for the calendar to open
    client.wait().for_element(Locator::Css("td.calendarbutton.calendar-nav:nth-of-type(2)")).await.context("failed to find previous month button")?;

    // Navigate to the previous month and select the first day
    client.find(Locator::Css("td.calendarbutton.calendar-nav:nth-of-type(2)"))
    .await.context("failed to find previous month button")?
    .click()
    .await.context("failed to click previous month button")?;

    //wait for the previous month to load
    client.wait().for_element(Locator::Css("td.day")).await.context("failed to find first day element")?;

    //select the day the code is called(every day 1)
    client.find(Locator::Css("td.day.selected")).await.context("failed to find first day element")?
    .click()
    .await.context("failed to click first day")?;

    client.find(Locator::Id("vDATAFIM_dp_trigger")).await.context("failed to find end date trigger element")?
    .click()
    .await.context("failed to click end date trigger")?;

    //BUG working with the second calender there is a need to specify the div.calendar:last-of-type
    //wait for the second calendar to open
    client.wait().for_element(Locator::Css("div.calendar:last-of-type td.calendarbutton.calendar-nav:nth-of-type(2)")).await.context("failed to find previous month button")?;

    // Navigate to the previous month 
    client.find(Locator::Css("div.calendar:last-of-type td.calendarbutton.calendar-nav:nth-of-type(2)"))
    .await.context("failed to find previous month button")?
    .click()
    .await.context("failed to click previous month button")?;

    //wait for the previous month to load
    client.wait().for_element(Locator::Css("div.calendar:last-of-type td.day")).await.context("failed to wait for find current day element")?;

    //BUG when i did div.calendar:last-of-type td.day:last-of-type it would not work
    //so we get all the td.days and go to the last one
    let days = client.find_all(Locator::Css("div.calendar:last-of-type td.day")).await.context("failed to find day elements")?;
    let last_day = days.last().context("failed to find last day element")?;
    last_day.click().await.context("failed to click last day")?;

    // Click the BUTTON1 to submit the form
    client.find(Locator::Id("BUTTON1")).await.context("failed to find submit button")?
    .click()
    .await.context("failed to click submit button")?;

    //esperar a listagem de notas fiscais aparecer
    client.wait().for_element(Locator::Id("vPROCESSAR_0001")).await.context("failed to find processar button")?;

    //processar as notas fiscais do mes,necessario para o download
    client.find(Locator::Id("vPROCESSAR_0001")).await.context("failed to find processar button")?
    .click()
    .await.context("failed to click processar button")?;

    sleep(Duration::from_secs(30)).await;

    //Esperar o download estar disponivel
    client.wait().for_element(Locator::Id("vDOWNLOAD_0001")).await.context("failed to find download button")?;

    //Realiza o download
    client.find(Locator::Id("vDOWNLOAD_0001")).await.context("failed to find download link")?
    .click()
    .await.context("failed to click download link")?;

    sleep(Duration::from_secs(5)).await;

    let month = chrono::Local::now().date_naive().month();
    let year = chrono::Local::now().year();

    let mut last_modified_file: Option<(PathBuf, SystemTime)> = None;

    let mut dir = read_dir("nota_fiscal/export_lotes/").await.context("failed to read dir")?;
    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await.context("Failed to read metadata")?;
        let modified_time = metadata.modified().context("Failed to get modified time")?;

        match last_modified_file {
            Some((_, last_modified_time)) => {
                if modified_time > last_modified_time {
                    last_modified_file = Some((entry.path(), modified_time));
                }
            }
            None => {
                last_modified_file = Some((entry.path(), modified_time));
            }
        }
    }

    // Check if we found a file and save the path to the database
    if let Some((last_file_path, _)) = last_modified_file.clone() {
        let last_file_path_str = last_file_path.to_str().context("Failed to convert path to string")?.to_string();
        query!("INSERT INTO nf_lote (path, month, year) VALUES ($1, $2, $3)", last_file_path_str, month as i32, year)
            .execute(pool)
            .await
            .context("Failed to save path to database")?;
    } else {
        println!("No files found in the directory.");
    }

    //Envia o lote de nota fiscal para a contabilidade
    //TODO pegar a quantidade de notas fiscais do lote
    send_nf_lote(pool, mailer, last_modified_file.unwrap().0,0)
    .await.context("Erro ao enviar email com lote de nota fiscal")?;

    Ok(())
}

async fn setup_gera_nf_chromedriver(nome: &str) -> Capabilities {
    // Define the download directory
    let download_dir = format!("/home/user/code/smart_gest/nota_fiscal/{}/", nome);
    //BUG this can be dangerous
    //TODO should get the current directory and append the download_dir to it,since the app run on its dir

    tokio::fs::create_dir_all(&download_dir).await.expect("failed to create download directory");

    // Create the preferences map
    let mut prefs = serde_json::Map::new();
    prefs.insert("savefile.default_directory".to_string(), json!(download_dir));
    prefs.insert("download.prompt_for_download".to_string(), json!(false));
    prefs.insert("directory_upgrade".to_string(), json!(true));

    // Create the chrome options map
    let mut chrome_options = serde_json::Map::new();
    chrome_options.insert("args".to_string(), json!(["--kiosk-printing"]));
    //TODO make it headless and etc
    //chrome_options.insert("args".to_string(), json!(["--headless=new,--disable-gpu,--no-sandbox,--disable-dev-shm-usage,--kiosk-printing"]));
    chrome_options.insert("prefs".to_string(), json!(prefs));

    // Create the capabilities map
    let mut caps = Capabilities::new();
    caps.insert("goog:chromeOptions".to_string(), json!(chrome_options));

    caps
}

pub async fn gera_nfs(pool:&PgPool,cliente:&ClienteNf,value:f32,mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,payment_id: i32) -> Result<(),anyhow::Error> {
    if cliente.gera_nf == false {
        return Ok(());
    }

    let caps = setup_gera_nf_chromedriver(&cliente.nome).await;

    let client = ClientBuilder::native()
    .capabilities(caps)
    .connect("http://localhost:9515").await.map_err(|e| {
        error!("failed to connect to WebDriver: {:?}", e);
        e
    }).expect("failed to connect to WebDriver");

    login(&client).await.context("Erro ao realizar login no sistema de nota fiscal")?;
    debug!("logged in for gerar nfs");

    client.wait().for_element(Locator::Css("td a[href='hwmemitenfse1_a24'] i.fa-pencil-square-o")).await.map_err(|e| {
        error!("failed to find gera nfs element: {:?}", e);
        e
    }).expect("failed to find element");

    let button = client.find(Locator::Css("td a[href='hwmemitenfse1_a24'] i.fa-pencil-square-o")).await.expect("failed to find element");
    button.click().await.map_err(|e| {
        error!("failed to click gera nfs element: {:?}", e);
        e
    }).expect("failed to click element");

    input_cliente(&client, cliente.cpf_cnpj.as_str()).await.context("Erro ao colocar cpf/cnpj do cliente")?;

    dados_nfs(&cliente, &client, value).await.context("Erro ao colocar dados da nota fiscal: endereco,valor servico,etc...")?;

    sleep(Duration::from_secs(5)).await;
    //Salva nota fisca com a data em nota fiscal/{cliente_nome}/{data}.pdf,retorna o caminho da mesma
    let nf = salvar_nota_fiscal(&client, &cliente.nome).await.context("Erro ao salvar nota fiscal")?;

    //BUG o fato de salvar depender de haver um email nao seria ideal
    if let Some(mailer) = mailer {
        //TODO create some kind of retry logic based on it,idk
        let sent = send_nf(&pool, &mailer, &cliente.email, nf.clone())
            .await.context("Erro ao enviar email com nota fiscal")?;

        query!("INSERT INTO nfs (path, payment_received_id, sent) VALUES ($1, $2, $3)", nf.as_str(), payment_id, sent)
            .execute(pool).await.context("Erro ao salvar nota fiscal no banco de dados")?;
    }
    Ok(())
}

//the chromedriver will save to nota_fiscal/{cliente_nome} already
//will need to get the last modified file and rename it to the date
async fn salvar_nota_fiscal(client:&Client,nome:&str) -> Result<String,anyhow::Error> {
    //BUG maybe this doesnt match id
    client.wait().for_element(Locator::Css("#BTNIMPRIMIR")).await.context("failed to find imprimir button")?;
    sleep(Duration::from_secs(5)).await;

    client.find(Locator::Css("#BTNIMPRIMIR")).await.context("failed to find imprimir button")?
    .click()
    .await.context("failed to click imprimir button")?;

    sleep(Duration::from_secs(10)).await;

    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
    let path = format!("nota_fiscal/{}/",nome);
    let mut dir = read_dir(&path).await.context("failed to read dir")?;

    // Variables to track the last modified file
    let mut last_modified_file: Option<(PathBuf, SystemTime)> = None;

    while let Some(entry) = dir.next_entry().await.context("Failed to get next entry")? {
        let metadata = entry.metadata().await.context("Failed to get metadata")?;
        let modified_time = metadata.modified().context("Failed to get modified time")?;

        match last_modified_file {
            Some((_, last_modified_time)) => {
                if modified_time > last_modified_time {
                    last_modified_file = Some((entry.path().clone(), modified_time));
                }
            }
            None => {
                last_modified_file = Some((entry.path().clone(), modified_time));
            }
        }

        // Rename the last modified file to include the current date
        if let Some((last_file_path, _)) = last_modified_file.clone() {
            let current_date = Local::now().format("%d_%m_%Y").to_string();
            let new_file_name = format!("{}/{}.pdf", path, current_date);
            rename(&last_file_path, &new_file_name).await.context("Failed to rename file")?;
            //tokio::fs::remove_file(last_file_path).await.context("Failed to remove old file")?;
            return Ok(new_file_name);
        } else {
            warn!("No files found in the directory.");
            anyhow::bail!("No files found in the directory")
        }
    }
    anyhow::bail!("No files found in the directory")
    //return the path to the file,for sending the email for the cliente
}

async fn input_cliente(client: &Client,cpf_cnpj: &str) -> Result<(),anyhow::Error> {    
    // Wait for and locate the CPF/CNPJ input element
    client.wait().for_element(Locator::Css("#vCTBCPFCNPJ")).await.context("failed to find cpf/cnpj input element")?;

    // Input the CPF/CNPJ value
    client.find(Locator::Css("#vCTBCPFCNPJ"))
    .await.context("failed to find cpf/cnpj input element")?
    .send_keys(cpf_cnpj)
    .await.context("failed to input cpf/cnpj value")?;

    client.find(Locator::Css("#vNFSLOCPRESTSRV"))
    .await.context("failed to find local prestacao de servico select element")?
    .click()
    .await.context("failed to click local prestacao de servico select element")?;
    
    client.wait().for_element(Locator::Css("#vNFSLOCPRESTSRV option[value='2']")).await.context("failed to find option element on local prestacao de servico")?;

    // Select the option by value
    client.find(Locator::Css("#vNFSLOCPRESTSRV option[value='2']")).await.context("failed to find option element on local prestacao de servico")?
        .click().await.context("failed to select option")?;

    // Locate and set "Municipio Prestador de Serviço" input element
    client.find(Locator::Css("#vNFSMUNICPRESTSER"))
    .await.context("failed to find municipio prestador de servico input element")?
    .send_keys(ID_MUNICIPIO)
    .await.context("failed to input municipio prestador de servico value")?;

    // Wait for and locate the Avançar button
    client.wait().for_element(Locator::Css("#BTNAVANCAR")).await.context("failed to find avancar button element")?;

    //Go to next page
    client.find(Locator::Css("#BTNAVANCAR")).await.context("failed to find avancar button element")?
    .click()
    .await.context("failed to click avancar button")?;

    Ok(())
}

async fn dados_nfs(cliente: &ClienteNf, client: &Client, value: f32) -> Result<(), anyhow::Error> {
    sleep(Duration::from_secs(1)).await;
    // Locate and set "Razão Social" if empty
    client.wait().for_element(Locator::Css("#vCTBRAZSOC")).await.context("failed to find Razão Social input element")?;

    let razao_social_element = client.find(Locator::Css("#vCTBRAZSOC")).await.context("failed to find Razão Social input element")?;
    let current_value = razao_social_element.prop("value").await.context("failed to get value of Razão Social input element")?;
    if current_value.is_none() || current_value.unwrap().is_empty() {
        razao_social_element.send_keys(&cliente.nome).await.context("failed to input Razão Social value")?;
    }

    sleep(Duration::from_secs(1)).await;
    // Locate and set "Nome Logradouro"
    client.wait().for_element(Locator::Css("#vNOMLOG")).await.context("failed to find nome logradouro input element")?;

    let nome_logradouro_element = client.find(Locator::Css("#vNOMLOG")).await.context("failed to find nome logradouro input element")?;
    let current_value = nome_logradouro_element.prop("value").await.context("failed to get value of nome logradouro input element")?;
    if current_value.is_none() || current_value.unwrap().is_empty() {
        nome_logradouro_element.send_keys(&cliente.rua).await.context("failed to input value in nome logradouro")?;
    }

    sleep(Duration::from_secs(1)).await;
    // Locate and set "Número" if it exists and is empty
    if let Some(numero) = &cliente.numero {
        let numero_element = client.find(Locator::Css("#vCTBENDNUMERO")).await.context("failed to find endereco numero input element")?;
        let current_value = numero_element.prop("value").await.context("failed to get value of endereco numero input element")?;
        if current_value.is_none() || current_value.unwrap().is_empty() {
            numero_element.send_keys(numero).await.context("failed to input value in endereco numero")?;
        }
    }

    sleep(Duration::from_secs(1)).await;
    // Locate and set "Complemento" if it exists and is empty
    if let Some(complemento) = &cliente.complemento {
        let complemento_element = client.find(Locator::Css("#vCTBCOMPLE")).await.context("failed to find complemento input element")?;
        let current_value = complemento_element.prop("value").await.context("failed to get value of complemento input element")?;
        if current_value.is_none() || current_value.unwrap().is_empty() {
            complemento_element.send_keys(complemento).await.context("failed to input value in complemento")?;
        }
    }

    sleep(Duration::from_secs(1)).await;
    // Locate and set "CEP"
    let cep_element = client.find(fantoccini::Locator::Css("#vCTBCEP")).await.context("failed to find cep input element")?;
    let current_value = cep_element.prop("value").await.context("failed to get value of cep input element")?;
    if current_value.is_none() || current_value.unwrap().is_empty() {
        cep_element.send_keys(&cliente.cep).await.context("failed to input value in cep")?;
    }

    sleep(Duration::from_secs(1)).await;
    // Locate and set "ID do Município"
    client.find(fantoccini::Locator::Css("#vMUNID")).await.context("failed to find municipio input element")?
        .send_keys(ID_MUNICIPIO).await.context("failed to input value in municipio")?;

    sleep(Duration::from_secs(1)).await;
    // Locate and set "Código do Serviço"
    client.find(Locator::Css("#vSRVSIGLA"))
        .await.context("failed to find sigla input element")?
        .send_keys(ID_SERVICO)
        .await.context("failed to input value in sigla")?;

    sleep(Duration::from_secs(1)).await;
    // Locate and clear "Valor do Serviço"
    client.find(Locator::Css("#vNFIVLRSRV"))
        .await.context("failed to find valor servico input element")?
        .clear().await.context("failed to clear valor servico input element")?;

    sleep(Duration::from_secs(1)).await;
    client.find(Locator::Css("#vNFIVLRSRV"))
        .await.context("failed to find valor servico input element")?
        .send_keys(&value.to_string().replace(".", ","))
        .await.context("failed to input value in valor servico")?;

    sleep(Duration::from_secs(1)).await;
    // Locate and click the "Add Service" button
    client.wait().for_element(Locator::Css("#vVIMGADDSRV")).await.context("failed to find add service button")?;
    client.execute("document.getElementById('vVIMGADDSRV').click();", vec![]).await.context("failed to click add service button using JavaScript")?;

    sleep(Duration::from_secs(1)).await;

    // Locate and set "Código CNAE", it's a select
    client.wait().for_element(Locator::Css("#vCOMBOCNAE")).await.context("failed to find cnae input element")?;
    client.find(Locator::Css("#vCOMBOCNAE option[value='3413']")).await.context("failed to find option element for CNAE code 3413")?
        .click().await.context("failed to select CNAE code 3413")?;

    sleep(Duration::from_secs(1)).await;
    // Locate and set "Código CNAE"
    client.find(Locator::Css("#vNBSCODIGO"))
        .await.context("failed to find codigo input element")?
        .send_keys(ID_CNAE)
        .await.context("failed to input value in codigo")?;

    sleep(Duration::from_secs(1)).await;
    // Locate and set "Descrição Geral do Serviço"
    let descricao_element = client.find(Locator::Css("#vNFSDSCGERAL")).await.context("failed to find descricao input element")?;
    let current_value = descricao_element.prop("value").await.context("failed to get value of descricao input element")?;
    if current_value.is_none() || current_value.unwrap().is_empty() {
        descricao_element.send_keys(DESCRICAO).await.context("failed to input value in descricao")?;
    }

    sleep(Duration::from_secs(1)).await;
    // Locate and click the "Visualizar Nota Fiscal" button
    client.wait().for_element(Locator::Css("#BUTTON3")).await.context("failed to find button element")?;
    client.find(Locator::Css("#BUTTON3")).await.context("failed to find button element")?
        .click().await.context("failed to click button")?;

    sleep(Duration::from_secs(30)).await;
    client.wait().for_element(Locator::Css("#BTNGERAR")).await.context("failed to find voltar button")?;
    client.find(Locator::Css("#BTNGERAR")).await.context("failed to find voltar button")?
        .click().await.context("failed to click voltar button")?;

    Ok(())
}

async fn login(client: &Client) -> Result<(),anyhow::Error> {
    // Navigate to the login page
    client.goto("https://e-nfs.com.br/e-nfs_novalima/servlet/hlogin").await.context("Failed to navigate to login page")?;

    // Wait for and locate the login input element
    client.wait().for_element(Locator::Id("vUSULOGIN")).await.context("failed to find cpf/cnpj login input")?;

    client.find(Locator::Id("vUSULOGIN")).await.context("failed to find login input")?
    // Enter the CNPJ value
    .send_keys(CNPJ).await.context("failed to enter CNPJ")?;

    // Wait for and locate the password input element
    //client.wait().for_element(Locator::Id("vSENHA")).await.context("failed to find password input")?;
    client.find(Locator::Id("vSENHA")).await.context("failed to find password input")?
    // Enter the password value
    .send_keys(PASSWORD).await.context("failed to enter password")?;

    // Locate and click the submit button
    client.wait().for_element(Locator::Id("BUTTON1")).await.context("failed to find submit button")?;

    client.find(Locator::Id("BUTTON1")).await.context("failed to find submit button")?
    .click().await.context("failed to click submit")?;

    Ok(())
}
