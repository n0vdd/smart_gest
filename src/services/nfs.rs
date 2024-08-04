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
use std::{path::PathBuf, time::SystemTime};

use fantoccini::{Client, ClientBuilder, Locator};
use tracing::{debug, error};

use crate::models::client::{Cliente, ClienteNf};


//TODO cancela nota fiscal
pub async fn cancela_nfs() -> Result<(),anyhow::Error> {
    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
        error!("failed to connect to WebDriver: {:?}", e);
        e
    }).expect("failed to connect to WebDriver");
    //TODO login no sistema da prefeitura de nova lima
    login(&client).await.context("erro ao logar no sistema de nota fiscal")?;


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

fn setup_chromedriver() -> serde_json::Map<String, serde_json::Value> {
    // Define the Chrome options with download preferences
    //BUG download dir e um caminho completo, quando mudar de servidor tenho que alterar aqui
    let download_dir = "/home/user/code/smart_gest/nota_fiscal/export_lotes/"; 

    let mut prefs = serde_json::Map::new();
    prefs.insert("download.default_directory".to_string(), serde_json::Value::String(download_dir.to_string()));

    let mut chrome_options = serde_json::Map::new();
    //prefs.insert("directory_upgrade".to_string(), serde_json::Value::Bool(true));
    chrome_options.insert("prefs".to_string(), serde_json::Value::Object(prefs));

    let mut caps = serde_json::Map::new();
    caps.insert("goog:chromeOptions".to_string(), serde_json::Value::Object(chrome_options));

    caps
}

pub async fn exporta_nfs(pool: &PgPool) -> Result<(),anyhow::Error> {
    let caps = setup_chromedriver();

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

    //TODO navegar para https://e-nfs.com.br/e-nfs_novalima/servlet/hwmcontabilidade

    //seleciona as datas no calendario de js

    //clica em BUTTON1

    //clica em vDOWNLOAD_0001

}

//TODO gera nota fiscal para os clientes que tiverem o pagamento confirmado
//TODO pegar os valores para o scraper usando f12
pub async fn gera_nfs(cliente:&Cliente,value:f32) {

    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
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

    dados_nfs(&cliente, &client, value).await;

    //TODO salvar a mesma para o sistema de arquivos
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
    Ok(())
}

//the chromedriver will save to nota_fiscal/{cliente_nome} already
//will need to get the last modified file and rename it to the date
async fn salvar_nota_fiscal(client:&Client,nome:&str) -> Result<(),anyhow::Error> {
    //BUG maybe this doesnt match id
    client.wait().for_element(Locator::Css("#BUTTON2.btnimprimir")).await.context("failed to find imprimir button")?;

    client.find(Locator::Css("#BUTTON2.btnimprimir")).await.context("failed to find imprimir button")?
    .click()
    .await.context("failed to click imprimir button")?;

    //maybe this will already save the pdf, if not will need to click on the save button
    client.wait().for_element(Locator::Css("cr-button.action-button")).await.context("failed to find salvar button")?;

    client.find(Locator::Css("cr-button.action-button")).await.context("failed to find salvar button")?
    .click()
    .await.context("failed to click salvar button")?;

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
            let current_date = Local::now().format("%d-%m-%Y").to_string();
            let new_file_name = format!("{}/{}.pdf", path, current_date);
            rename(&last_file_path, &new_file_name).await.context("Failed to rename file")?;
        } else {
            warn!("No files found in the directory.");
        }
    }

    Ok(())
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

async fn dados_nfs(cliente:&Cliente,client: &Client,value:f32) {
    client.wait().for_element(Locator::Css("#vCTBRAZSOC")).await.map_err(|e| {
        error!("failed to find Razão Social input element: {:?}", e);
        e
    }).expect("failed to find Razão Social input element");
    // Locate the "Razão Social" input element by its ID
    let razao_social_element = client.find(Locator::Css("#vCTBRAZSOC"))
    .await.map_err(|e| {
        error!("failed to find Razão Social input element: {:?}", e);
        e
    })
    .expect("failed to find Razão Social input element");

    // Caso nao se tenha a razao social
    let current_value = razao_social_element.prop("value")
    .await.map_err(|e| {
        error!("failed to get value of Razão Social input element: {:?}", e);
        e
    }).expect("failed to get value of Razão Social input element");

    //Seta com o nome do cliente
    if current_value.is_none() {
        nome_logradouro_element.send_keys(&cliente.rua).await.context("failed to input value in nome logradouro")?;
    }

       // Locate and set "Número" if it exists and is empty
    if let Some(numero) = &cliente.numero {
        //client.wait().for_element(Locator::Css("#vCTBENDNUMERO")).await.context("failed to find endereco numero input element
        let numero_element = client.find(Locator::Css("#vCTBENDNUMERO")).await.context("failed to find endereco numero input element")?;
        let current_value = numero_element.prop("value").await.context("failed to get value of endereco numero input element")?;
        if current_value.is_none() {
            numero_element.send_keys(numero).await.context("failed to input value in endereco numero")?;
        }
    }

    // Locate and set "Complemento" if it exists and is empty
    if let Some(complemento) = &cliente.complemento {
        //client.wait().for_element(Locator::Css("#vCTBCOMPLE")).await.context("failed to find complemento input element")?;
        let complemento_element = client.find(Locator::Css("#vCTBCOMPLE")).await.context("failed to find complemento input element")?;
        let current_value = complemento_element.prop("value").await.context("failed to get value of complemento input element")?;
        if current_value.is_none() {
            complemento_element.send_keys(complemento).await.context("failed to input value in complemento")?;
        }
    }

    // Locate and set "CEP"
    //client.wait().for_element(Locator::Css("#vCTBCEP")).await.context("failed to find cep input element")?;
    let cep_element = client.find(Locator::Css("#vCTBCEP")).await.context("failed to find cep input element")?;
    let current_value = cep_element.prop("value").await.context("failed to get value of cep input element")?;
    if current_value.is_none() {
        cep_element.send_keys(&cliente.cep).await.context("failed to input value in cep")?;
    }

    // Locate and set "ID do Município"
    //client.wait().for_element(Locator::Css("#vMUNID")).await.context("failed to find municipio input element")?;
    let municipio_element = client.find(Locator::Css("#vMUNID")).await.context("failed to find municipio input element")?;
    let current_value = municipio_element.prop("value").await.context("failed to get value of municipio input element")?;
    if current_value.is_none() {
        municipio_element.send_keys(ID_MUNICIPIO).await.context("failed to input value in municipio")?;
    }

    // Locate and set "Email" (without checking for existing value)
    //client.wait().for_element(Locator::Css("#vCTBEMAIL")).await.context("failed to find email input element")?;
    client.find(Locator::Css("#vCTBEMAIL")).await.context("failed to find email input element")?
        .send_keys(&cliente.email).await.context("failed to input value in email field")?;

    // Locate and set "Código do Serviço"
    //client.wait().for_element(Locator::Css("#vSRVSIGLA")).await.context("failed to find sigla input element")?;
    let servico_element = client.find(Locator::Css("#vSRVSIGLA")).await.context("failed to find sigla input element")?;
    let current_value = servico_element.prop("value").await.context("failed to get value of sigla input element")?;
    if current_value.is_none() {
        servico_element.send_keys(ID_SERVICO).await.context("failed to input value in sigla")?;
    }

    // Locate and set "Valor do Serviço"
    //client.wait().for_element(Locator::Css("#vNFIVLRSRV")).await.context("failed to find valor servico input element")?;
    let valor_servico_element = client.find(Locator::Css("#vNFIVLRSRV")).await.context("failed to find valor servico input element")?;
    let current_value = valor_servico_element.prop("value").await.context("failed to get value of valor servico input element")?;
    if current_value.is_none() {
        valor_servico_element.send_keys(&value.to_string()).await.context("failed to input value in valor servico")?;
    }

    // Locate and click the "Add Service" button
    client.wait().for_element(Locator::Css("#vIMGADDSRV")).await.context("failed to find add service button")?;
    client.find(Locator::Css("#vIMGADDSRV")).await.context("failed to find add service button")?
        .click().await.context("failed to click add service button")?;

    // Locate and set "Código CNAE"
    //client.wait().for_element(Locator::Css("#vNBSCODIGO")).await.context("failed to find codigo input element")?;
    let codigo_element = client.find(Locator::Css("#vNBSCODIGO")).await.context("failed to find codigo input element")?;
    let current_value = codigo_element.prop("value").await.context("failed to get value of codigo input element")?;
        if current_value.is_none() {
        codigo_element.send_keys(ID_CNAE).await.context("failed to input value in codigo")?;
    }

    // Locate and set "Descrição Geral do Serviço"
    //client.wait().for_element(Locator::Css("#vNFSDSCGERAL")).await.context("failed to find descricao input element")?;
    let descricao_element = client.find(Locator::Css("#vNFSDSCGERAL")).await.context("failed to find descricao input element")?;
    let current_value = descricao_element.prop("value").await.context("failed to get value of descricao input element")?;
    if current_value.is_none() {
        descricao_element.send_keys(DESCRICAO).await.context("failed to input value in descricao")?;
    }

    // Locate and click the "Visualizar Nota Fiscal" button
    client.wait().for_element(Locator::Css("#BUTTON3")).await.context("failed to find button element")?;
    client.find(Locator::Css("#BUTTON3")).await.context("failed to find button element")?
        .click().await.context("failed to click button")?;

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
