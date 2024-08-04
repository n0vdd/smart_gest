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


use std::{path::PathBuf, time::SystemTime};

use anyhow::Context;
use chrono::Local;
use fantoccini::{Client, ClientBuilder, Locator};
use tokio::fs::{create_dir, create_dir_all, read_dir, rename};
use tracing::{debug, error, warn};

use crate::models::client::{Cliente, ClienteNf};


//TODO cancela nota fiscal
pub async fn cancela_nfs() {
    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
        error!("failed to connect to WebDriver: {:?}", e);
        e
    }).expect("failed to connect to WebDriver");
    //TODO login no sistema da prefeitura de nova lima
    login(&client).await;


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
}

//TODO recebe o mes/ano que estamos
//usa o mes que estamos como data final(sempe chamado no inicio do mes)
//e o mes passado como o mes inicial
pub async fn exporta_nfs(month:i32,year:i32) {
    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
        error!("failed to connect to WebDriver: {:?}", e);
        e
    }).expect("failed to connect to WebDriver");

    login(&client).await;

    //TODO navegar para https://e-nfs.com.br/e-nfs_novalima/servlet/hwmcontabilidade

    //seleciona as datas no calendario de js

    //clica em BUTTON1

    //clica em vDOWNLOAD_0001

}

//TODO set download dir for chromedriver
async fn setup_chromedriver_gera_nfs(nome:&str) -> serde_json::Map<String,serde_json::Value> {
    let download_dir = format!("/home/user/code/smart_gest/nota_fiscal/{}/",nome);
    //BUG check the path, you dont want to create /home/user/code/smart_gest just the nota_fiscal/{nome}
    //but there is a need to specify the full path for chromedriver
    create_dir_all(&download_dir).await.expect("Erro ao criar diretorio de download");

    let mut caps = serde_json::Map::new();
    //TODO should do this for all the clients
    caps.insert("goog:chromeOptions".to_string(), serde_json::json!({
        //"args": ["--headless", "--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"],
        "prefs": {
            "download.default_directory": download_dir,
            "download.prompt_for_download": false,
            "download.directory_upgrade": true,
            "safebrowsing.enabled": true
        }
    }));
    caps
}

//TODO gera nota fiscal para os clientes que tiverem o pagamento confirmado
//TODO pegar os valores para o scraper usando f12
pub async fn gera_nfs(cliente:&ClienteNf,value:f32) {
    if cliente.gera_nf == false {
        return;
    }
    let caps = setup_chromedriver_gera_nfs(&cliente.nome).await;
    let client = ClientBuilder::native()
    .capabilities(caps)
    .connect("http://localhost:9515").await.map_err(|e| {
        error!("failed to connect to WebDriver: {:?}", e);
        e
    }).expect("failed to connect to WebDriver");

    //TODO login no sistema da prefeitura de nova lima

    //TODO clicar no link de gerar nota fiscal no canto direito 

    //TODO preencher os campos com os dados necessarios(alguns podem ser hardcoded?)
    //? talvez tenha que refazer a estrutura de dados dos planos para incluir as coisas fiscais
    //ai pego tudo pelo cliente(plano esta relacionado ao cliente entao fica facil)


    login(&client).await;
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

    input_cliente(&client, cliente.cpf_cnpj.as_str()).await;

    dados_nfs(&cliente, &client, value).await;

    salvar_nota_fiscal(&client, &cliente.nome).await.expect("failed to save nota fiscal");
    //salvar nota fiscal,should change the name, but since i dont ask for the path i will need to rename it i think

    //enviar e rececer a nota fiscal
    //salvar a mesma para o sistema de arquivos
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
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

async fn input_cliente(client: &Client,cpf_cnpj: &str) {
    client.wait().for_element(Locator::Css("#vCTBCPFCNPJ")).await.map_err(|e| {
        error!("failed to find cpf/cnpj input element: {:?}", e);
        e
    }).expect("failed to find input element");

    // Locate the input element by its ID and input a value
    client.find(Locator::Css("#vCTBCPFCNPJ"))
    .await.map_err(|e| {
        error!("failed to find cpf/cnpj input element: {:?}", e);
        e
    }).expect("failed to find input element")
    //TODO this should be unformatted cpf/cnpj
    .send_keys(cpf_cnpj)
    .await.map_err(|e| {
        error!("failed to input cpf/cnpj: {:?}", e);
        e
    }).expect("failed to input cpf/cpnj value");

    // Locate the select element by its ID and select an option by its value
    let select_element = client.find(Locator::Css("#vNFSLOCPRESTSRV")).await.map_err(|e| {
        error!("failed to find local prestacao de servico select element: {:?}", e);
        e
    }).expect("failed to find select element");

    select_element.click().await.map_err(|e| {
        error!("failed to click local prestacao de servico select element: {:?}", e);
        e
    }).expect("failed to click select element");

    // Select the option by value
    client.find(Locator::Css("#vNFSLOCPRESTSRV option[value='2']"))
    .await.expect("failed to find option element on local prestacao de servico")
    .click()
    .await.expect("failed to select option");

    client.find(Locator::Css("#vNFSMUNICPRESTSER"))
    .await.map_err(|e| {
        error!("failed to find municipio prestador de servico input element: {:?}", e);
        e
    }).expect("failed to find input element")
    .send_keys(ID_MUNICIPIO)
    .await.map_err(|e| {
        error!("failed to input municipio prestador de servico: {:?}", e);
        e
    }).expect("failed to input municipio prestador de servico value");

    client.find(Locator::Css("#BTNAVANCAR"))
    .await.map_err(|e| {
        error!("failed to find avancar button element: {:?}", e);
        e
    }).expect("failed to find button element")
    .click()
    .await.map_err(|e| {
        error!("failed to click avancar button: {:?}", e);
        e
    }).expect("failed to click button");
}

async fn dados_nfs(cliente:&ClienteNf,client: &Client,value:f32) {
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
    razao_social_element.send_keys(&cliente.nome)
        .await.map_err(|e| {
            error!("failed to input Razão Social: {:?}", e);
            e
        }).expect("failed to input Razão Social value");
    }

    client.wait().for_element(Locator::Css("#vNOMLOG")).await.map_err(|e| {
        error!("failed to find Razão Social input element: {:?}", e);
        e
    }).expect("failed to find Razão Social input element");

    // Seta o nome do logradouro
    client.find(Locator::Css("#vNOMLOG"))
    .await.map_err(|e| {
        error!("failed to find nome logradouro input element: {:?}", e);
        e
    })
    .expect("failed to find nome logradouro input element")
    .send_keys(&cliente.rua)
    .await.map_err(|e| {
        error!("failed to input value in nome logradouro: {:?}", e);
        e
    })
    .expect("failed to input value in vNOMLOG");


    //Seta o numero caso ele exista
    if let Some(numero) = &cliente.numero {
        // Fill in the vCTBENDNUMERO input field
        client.find(Locator::Css("#vCTBENDNUMERO"))
        .await.map_err(|e| {
            error!("failed to find endereco numero input element: {:?}", e);
            e
        })
        .expect("failed to find endereco numero input element")
        .send_keys(numero.as_ref())
        .await.map_err(|e| {
            error!("failed to input value in endereco numero: {:?}", e);
            e
        }).expect("failed to input value in endereco numero");
    }
    
    //Seta o complemento caso ele exista
    if let Some(complement) = &cliente.complemento {
        client.find(Locator::Css("#vCTBCOMPLE")).await.map_err(|e| {
            error!("failed to find complemento input element: {:?}", e);
            e
        }).expect("failed to find complemento input element")
        .send_keys(&complement).await.map_err(|e| {
            error!("failed to input value in complemento: {:?}", e);
            e
        }).expect("failed to input value in complemento");
    }

    //Seta o cep(de acordo com o endereco do cliente) 
    client.find(Locator::Css("#vCTBCEP"))
    .await.map_err(|e| {
        error!("failed to find cep input element: {:?}", e);
        e
    })
    .expect("failed to find cep input element")
    .send_keys(&cliente.cep)
    .await.map_err(|e| {
        error!("failed to input value in cep: {:?}", e);
        e
    })
    .expect("failed to input value in cep");

    //Seta a id do municipio(fixo)
    client.find(Locator::Css("#vMUNID"))
    .await.map_err(|e| {
        error!("failed to find municipio input element: {:?}", e);
        e
    }).expect("failed to find municipio input element").
    send_keys(ID_MUNICIPIO).await.map_err(|e| {
        error!("failed to input value in municipio: {:?}", e);
        e
    }).expect("failed to input value in municipio");

    //Seta o codigo do servico(fixo)
    client.find(Locator::Css("#vSRVSIGLA"))
    .await.map_err(|e| {
        error!("failed to find sigla input element: {:?}", e);
        e
    }).expect("failed to find sigla input element")
    .send_keys(ID_SERVICO).await.map_err(|e| {
        error!("failed to input value in sigla: {:?}", e);
        e
    }).expect("failed to input value in sigla");

    //Seta o valor do servico de acordo com o plano do cliente
    client.find(Locator::Css("#vNFIVLRSRV"))
    .await.map_err(|e| {
        error!("failed to find valor servico input element: {:?}", e);
        e
    }).expect("failed to find valor servico input element")
    .send_keys(&value.to_string()).await.map_err(|e| {
        error!("failed to input value in valor servico: {:?}", e);
        e
    }).expect("failed to input value in valor servico");

    client.wait().for_element(Locator::Css("#vIMGADDSRV")).await.map_err(|e| {
        error!("failed to find add service button: {:?}", e);
        e
    }).expect("failed to find add service button");

    client.find(Locator::Css("#vIMGADDSRV")).await.map_err(|e| {
        error!("failed to find add service button: {:?}", e);
        e
    }).expect("failed to find add service button")
    .click().await.map_err(|e| {
        error!("failed to click add service button: {:?}", e);
        e
    }).expect("failed to click add service button");

    //Codigo CNAE sempre sera o mesmo
    client.find(Locator::Css("#vNBSCODIGO"))
    .await.map_err(|e| {
        error!("failed to find codigo input element: {:?}", e);
        e
    }).expect("failed to find codigo input element")
    .send_keys(ID_CNAE).await.map_err(|e| {
        error!("failed to input value in codigo: {:?}", e);
        e
    }).expect("failed to input value in codigo");

    //Descricao geral do servico
    client.find(Locator::Css("#vNFSDSCGERAL"))
    .await.map_err(|e| {
        error!("failed to find descricao input element: {:?}", e);
        e
    }).expect("failed to find descricao input element")
    .send_keys(DESCRICAO).await.map_err(|e| {
        error!("failed to input value in descricao: {:?}", e);
        e
    }).expect("failed to input value in descricao");

    //Clilca para visualizar a nota fiscal
    client.find(Locator::Css("#BUTTON3")).await.map_err(|e| {
        error!("failed to find button element: {:?}", e);
        e
    }).expect("failed to find button element")
    .click().await.map_err(|e| {
        error!("failed to click button: {:?}", e);
        e
    }).expect("failed to click button");
}

// Example usage
async fn login(client: &Client) {
    client.goto("https://e-nfs.com.br/e-nfs_novalima/servlet/hlogin").await.map_err(|e| {
        error!("Failed to navigate to login page: {:?}", e);
        //? talvez nao deva parar a aplicacao,
        //mas tenho que que fazer um drama
        panic!("Failed to navigate to login page")
    }).expect("Failed to navigate to login page");

    client.wait().for_element(Locator::Id("vUSULOGIN")).await.map_err(|e| {
        error!("failed to find cpf/cnpj login input: {:?}", e);
        e
    }).expect("failed to find login input");

    client.find(Locator::Id("vUSULOGIN")).await.map_err(|e| { 
        error!("failed to find login input: {:?}", e);
        e
    }).expect("failed to find login input")
    .send_keys(CNPJ).await.map_err(|e| {
        error!("failed to enter CNPJ: {:?}", e);
        e
    }).expect("failed to enter CNPJ");

    client.wait().for_element(Locator::Id("vSENHA")).await.map_err(|e| {
        error!("failed to find password input: {:?}", e);
        e
    }).expect("failed to find password input");

    client.find(Locator::Id("vSENHA")).await.map_err(|e| {
        error!("failed to find password input: {:?}", e);
        e
    }).expect("failed to find password input").send_keys(PASSWORD).await.map_err(|e| {
        error!("failed to enter password: {:?}", e);
        e
    }).expect("failed to enter password");

    client.find(Locator::Id("BUTTON1")).await.map_err(|e| {
        error!("failed to find submit button: {:?}", e);
        e
    }).expect("failed to find submit button").click().await.map_err(|e| {
        error!("failed to click submit: {:?}", e);
        e
    }).expect("failed to click submit");
}
