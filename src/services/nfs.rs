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


use fantoccini::{Client, ClientBuilder, Locator};
use tracing::{debug, error};

use crate::handlers::clients::Cliente;

//TODO cancela nota fiscal


//TODO gera nota fiscal para os clientes que tiverem o pagamento confirmado
//TODO pegar os valores para o scraper usando f12
pub async fn gera_nfs(cliente:&Cliente,value:f32) {

    let client = ClientBuilder::native().connect("http://localhost:9515").await.map_err(|e| {
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

    //enviar e rececer a nota fiscal
    //salvar a mesma para o sistema de arquivos
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
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