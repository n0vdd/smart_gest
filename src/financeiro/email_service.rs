    
use std::path::PathBuf;

use anyhow::Context;
use chrono::{Datelike, Local};
use lettre::{message::{header, Attachment, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx:: PgPool;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{config::config::{find_email_config, get_email_used_in_config, get_nf_config_emails, get_nome_used_in_provedor}, TEMPLATES};


//this should be called by the function who saves the email config
//or maybe all function that send mail will instantiate the email
//this would not be optimal for sending 150 nfs one after another
pub async fn setup_email(pool: &PgPool) -> Result<AsyncSmtpTransport<Tokio1Executor>,anyhow::Error> {
    let mail_config = find_email_config(pool).await.context("Erro ao buscar a configuração de email")?.expect("Email config not found");

    let creds = Credentials::new(mail_config.email, mail_config.password);

    //Use ssl by default, so the server should support it
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&mail_config.host)?
        .credentials(creds)
        .build();

    info!("Email para envio: {:?}", mailer);

    mailer.test_connection().await.context("Erro ao testar conexão com o email")?;

    Ok(mailer)
}

pub async fn send_nf(pool:&PgPool, mailer: &AsyncSmtpTransport<Tokio1Executor>, to: &str, nf:String) -> Result<bool,anyhow::Error> {
    let mut file = File::open(&nf).await?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).await?;

    let from_mail = get_email_used_in_config(pool).await?; 

    let nome_provedor = get_nome_used_in_provedor(pool).await?; 

    let attachment = Attachment::new("nota_fiscal.pdf".to_string())
        .body(file_content, "application/pdf".parse()?);

    let data = Local::now().format("%d/%m/%Y").to_string();
    let mes_atual = Local::now().month();
    let ano_atual = Local::now().year();

    //Como a cobranca ocorre dia 5, contamos de dia 5 a dia 5
    let inicio = format!("05/{}/{}",mes_atual,ano_atual);

    //BUG This will always be assigned month goes from 1 to 12 and i cover all the cases 
    let mut fim = String::new();
    if mes_atual == 12 {
        fim = format!("05/01/{}",ano_atual+1);
    } else {
        fim = format!("05/{}/{}",mes_atual+1 ,ano_atual);
    }
    //cobramos do mes atual
    //Sendo feito como se a nota fiscal emitida em novembro fosse referente ao servico de novembro a dezembro
    let periodo = format!("{} - {}", inicio, fim);
    let mut context = tera::Context::new();
    context.insert("data",&data );
    context.insert("periodo", &periodo);
    context.insert("nome_provedor",&nome_provedor);

    let body = TEMPLATES.render("nf_email.html", &context).context("Erro ao renderizar corpo do email")?;
    let email = Message::builder()
        .from(from_mail.parse().expect("Failed to parse sender email"))
        .to(to.parse().expect("Failed to parse cliente email"))
        .subject("Envio de Nota Fiscal - Serviço Prestado")
        .multipart(   
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(body),
                )
                .singlepart(attachment))?; 

    mailer.send(email).await.context("Erro ao enviar email")?;

    //If mailer send dont fail returns true
    Ok(true)
}

//TODO talves devesse ser enviado dia 5, fechar e comecar tudo de um dia 5 ao outro
pub async fn send_nf_lote(pool: &PgPool,mailer: &AsyncSmtpTransport<Tokio1Executor>,lote:PathBuf) -> Result<(),anyhow::Error> {
    //could already return the emails
    let emails = get_nf_config_emails(&pool).await?;    

    let mut file = File::open(&lote).await?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).await?;

    let attachment = Attachment::new("nf.zip".to_string())
        .body(file_content, "application/zip".parse()?);

    let from_mail = get_email_used_in_config(pool).await?; 

    let nome_provedor = get_nome_used_in_provedor(pool).await?; 

    let ano = Local::now().year();
    let mut next_ano = ano;
    let mes = Local::now().month();

    let subject = format!("Lote de Notas Fiscais - {}/{}",mes,ano);

    let mut next_mes = mes;
    if mes == 12 {
        //BUG caso fosse o mes de dezembro, o mes seguinte seria janeiro do ano que vem
        //logo teria que avancar o ano e voltar o mes 
        next_mes = 1;
        next_ano += 1;
    } else {
        next_mes += 1;
    }

    let periodo = format!("05/{}/{} - 05/{}/{}",mes,ano,next_mes,next_ano);

    let mut context = tera::Context::new();
    context.insert("nome",&nome_provedor);
    context.insert("mes", &mes);
    context.insert("ano", &ano);
    //context.insert("quantidade_notas", &qnt_nfs);
    context.insert("periodo", &periodo);


    let body = TEMPLATES.render("nf_lote_email.html", &context).context("Erro ao renderizar corpo do email")?;
    let email = Message::builder()
        .from(from_mail.parse().context("Failed to parse sender email")?)
        .to(emails.join(",").parse().context("Failed to parse cliente email")?)
        .subject(subject)
        .multipart(   
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(body),
                )
                .singlepart(attachment))?; 

    mailer.send(email).await.context("Erro ao enviar email")?;

    Ok(())
}