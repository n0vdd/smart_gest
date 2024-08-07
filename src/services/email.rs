
use std::path::PathBuf;

use anyhow::Context;
use chrono::{Datelike, Local};
use lettre::{message::{header, Attachment, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::{ query, query_as, PgPool};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{models::config::{EmailConfig, NfConfig}, TEMPLATES};

//this should be called by the function who saves the email config
//or maybe all function that send mail will instantiate the email
//this would not be optimal for sending 150 nfs one after another
pub async fn setup_email(pool: &PgPool) -> Result<AsyncSmtpTransport<Tokio1Executor>,anyhow::Error> {
    let mail_config = query_as!(EmailConfig,"SELECT * FROM email_config").fetch_one(&*pool).await
    .context("Erro ao buscar a configuração de email")?;

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

    let from_mail = query!("SELECT email FROM email_config").fetch_one(pool).await
        .context("Erro ao buscar o email de envio")?.email;

    let nome_provedor = query!("SELECT nome FROM provedor").fetch_one(pool).await
        .context("Erro ao buscar o nome do provedor")?.nome;

    let attachment = Attachment::new("nota_fiscal.pdf".to_string())
        .body(file_content, "application/pdf".parse()?);

    let data = Local::now().format("%d/%m/%Y").to_string();
    let mes_atual = Local::now().month();
    let ano_atual = Local::now().year();

    //Como a cobranca ocorre dia 5, contamos de dia 5 a dia 5
    let inicio = format!("05/{}/{}",mes_atual-1,ano_atual);
    let fim = format!("05/{}/{}",mes_atual ,ano_atual);
    //TODO cobramos do mes anterior ou do mes atual?
    //Sendo feito como se a nota fiscal emitida em novembro fosse referente ao servico de outubro a novembro
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
pub async fn send_nf_lote(pool: &PgPool,mailer: &AsyncSmtpTransport<Tokio1Executor>,lote:PathBuf,qnt_nfs:i32) -> Result<(),anyhow::Error> {
    let emails = query_as!(NfConfig,"SELECT * FROM nf_config").fetch_one(pool).await
        .context("Erro ao buscar os emails da contabilidade")?;

    let mut file = File::open(&lote).await?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).await?;

    let attachment = Attachment::new("nf.zip".to_string())
        .body(file_content, "application/zip".parse()?);

    let from_mail = query!("SELECT email FROM email_config").fetch_one(pool).await
        .context("Erro ao buscar o email de envio")?.email;

    let nome_provedor = query!("SELECT nome FROM provedor").fetch_one(pool).await
        .context("Erro ao buscar o nome do provedor")?.nome;

    let mut ano = Local::now().year();
    let mut mes = Local::now().month();

    //BUG caso fosse o mes de janeiro, o mes anterior seria dezembro do ano passado
    //logo teria que voltar o ano tambem
    if mes == 1 {
        ano = ano - 1;
        mes = 12;
    //caso contrato estamos nos referindo ao mes anterior pois enviamos o lote apos a virada do mes de referencia do mesmo
    } else {
        mes = mes - 1;
    }

    let subject = format!("Lote de Notas Fiscais - {}/{}",mes,ano);
    //BUG no caso da virada de ano tenho que atualizar o ano na seguna parte do periodo
    let periodo = format!("05/{}/{} - 05/{}/{}",mes,ano,mes+1,ano);

    let mut context = tera::Context::new();
    context.insert("nome",&nome_provedor);
    context.insert("mes", &mes);
    context.insert("ano", &ano);
    //TODO pegar essa informacao da pagina apos processar o lote
    context.insert("quantidade_notas", &qnt_nfs);
    context.insert("periodo", &periodo);


    let body = TEMPLATES.render("nf_lote_email.html", &context).context("Erro ao renderizar corpo do email")?;
    let email = Message::builder()
        .from(from_mail.parse().context("Failed to parse sender email")?)
        .to(emails.contabilidade_email.join(",").parse().context("Failed to parse cliente email")?)
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