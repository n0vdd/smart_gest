//TODO automatizar criacao de nota fiscal de servico
//pelo site da prefeitura de nova lima
//talvez tenha como enviar por xml(mais dificil)
//? devo precisar de alguma crate para xml(tenho alguns docs sobre)
//talvez precisa auatomatizar como se fosse uma pessoa(eles nao devem verificar bots, so nao posso derrubar o site)

//enviar um email com a nota gerada apos pagamento


const CNPJ: &str = "48530335000148";
const PASSWORD: &str = "oU6jlxL7RpUY7JB3TAqD";
const ID_MUNICIPIO: &str = "17428";

use std::time::Duration;

use fantoccini::{Client, ClientBuilder, Locator};
use tracing::error;

use crate::handlers::clients::Cliente;


//TODO gera nota fiscal para os clientes que tiverem o pagamento confirmado
//TODO pegar os valores para o scraper usando f12
pub async fn gera_nfs(cliente:Cliente) {

    let client = ClientBuilder::native().connect("http://localhost:4444").await.map_err(|e| {
        error!("failed to connect to WebDriver: {:?}", e);
        e
    }).expect("failed to connect to WebDriver");

    //TODO login no sistema da prefeitura de nova lima

    //TODO clicar no link de gerar nota fiscal no canto direito 

    //TODO preencher os campos com os dados necessarios(alguns podem ser hardcoded?)
    //? talvez tenha que refazer a estrutura de dados dos planos para incluir as coisas fiscais
    //ai pego tudo pelo cliente(plano esta relacionado ao cliente entao fica facil)

    login(&client).await;

    let button = client.find(Locator::Css("td a[href='hwmemitenfse1_a24'] i.fa-pencil-square-o")).await.expect("failed to find element");
    button.click().await.map_err(|e| {
        error!("failed to click gera nfs element: {:?}", e);
        e
    }).expect("failed to click element");

    input_cliente(&client, cliente.cpf_cnpj.as_str()).await;
    

    //enviar e rececer a nota fiscal
    //salvar a mesma para o sistema de arquivos
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
}

async fn input_cliente(client: &Client,cpf_cnpj: &str) {
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
    .send_keys("17428")
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

async fn dados_nfs(cliente:&Cliente,client: &Client) {
    // Locate the "Razão Social" input element by its ID
    let razao_social_element = client.find(Locator::Css("#vCTBRAZSOC"))
    .await.map_err(|e| {
        error!("failed to find Razão Social input element: {:?}", e);
        e
    })
    .expect("failed to find Razão Social input element");

    // Check if the field is filled, if not input the name of the client
    let current_value = razao_social_element.prop("value")
    .await.map_err(|e| {
        error!("failed to get value of Razão Social input element: {:?}", e);
        e
    }).expect("failed to get value of Razão Social input element");

    if current_value.is_none() {
    razao_social_element.send_keys(&cliente.nome)
        .await.map_err(|e| {
            error!("failed to input Razão Social: {:?}", e);
            e
        }).expect("failed to input Razão Social value");
    }

    // Fill in the vNOMLOG input field
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

    // Fill in the vCTBENDNUMERO input field
    client.find(Locator::Css("#vCTBENDNUMERO"))
    .await.map_err(|e| {
        error!("failed to find endereco numero input element: {:?}", e);
        e
    })
    .expect("failed to find endereco numero input element")
    .send_keys("2287")
    .await.map_err(|e| {
        error!("failed to input value in endereco numero: {:?}", e);
        e
    }).expect("failed to input value in endereco numero");

    // Fill in the vCTBCOMPLE input field
    client.find(Locator::Css("#vCTBCOMPLE"))
    .await.map_err(|e| {
        error!("failed to find complemento input element: {:?}", e);
        e
    })
    .expect("failed to find complemento input element")
    .send_keys("SALA 810")
    .await.map_err(|e| {
        error!("failed to input value in complemento: {:?}", e);
        e
    })
    .expect("failed to input value in complemento");

    // Fill in the vCTBCEP input field
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
}
// Example usage
async fn login(client: &Client) {
    client.goto("https://e-nfs.com.br/e-nfs_novalima/servlet/hlogin").await.map_err(|e| {
        error!("Failed to navigate to login page: {:?}", e);
        //? talvez nao deva parar a aplicacao,
        //mas tenho que que fazer um drama
        panic!("Failed to navigate to login page")
    }).expect("Failed to navigate to login page");

    client.wait().for_element(Locator::XPath("//input[@placeholder='CPF/CNPJ do Prestador']")).await.map_err(|e| {
        error!("failed to find cpf/cnpj login input: {:?}", e);
        e
    }).expect("failed to find login input");

    client.find(Locator::XPath("//input[@placeholder='CPF/CNPJ do Prestador']")).await.map_err(|e| { 
        error!("failed to find login input: {:?}", e);
        e
    }).expect("failed to find login input")
    .send_keys(CNPJ).await.map_err(|e| {
        error!("failed to enter CNPJ: {:?}", e);
        e
    }).expect("failed to enter CNPJ");

    client.wait().for_element(Locator::XPath("//input[@placeholder='Senha']")).await.map_err(|e| {
        error!("failed to find password input: {:?}", e);
        e
    }).expect("failed to find password input");

    client.find(Locator::XPath("//input[@placeholder='Senha']")).await.map_err(|e| {
        error!("failed to find password input: {:?}", e);
        e
    }).expect("failed to find password input").send_keys(PASSWORD).await.map_err(|e| {
        error!("failed to enter password: {:?}", e);
        e
    }).expect("failed to enter password");

    client.find(Locator::XPath("//form//button[@type='submit']")).await.map_err(|e| {
        error!("failed to find submit button: {:?}", e);
        e
    }).expect("failed to find submit button").click().await.map_err(|e| {
        error!("failed to click submit: {:?}", e);
        e
    }).expect("failed to click submit");
}