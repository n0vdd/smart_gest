
use std::path::PathBuf;

use anyhow::Context;
use chrono::{Datelike, Local};
use lettre::{message::{header, Attachment, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::{ query, query_as, PgPool};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{models::config::EmailConfig, TEMPLATES};

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

    Ok(mailer)
}

pub async fn send_nf(pool:&PgPool, mailer: &AsyncSmtpTransport<Tokio1Executor>, to: &str, nf:PathBuf,nome:&str,valor:f32) -> Result<(),anyhow::Error> {
    let mut file = File::open(&nf).await?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).await?;

    let from_mail = query!("SELECT email FROM email_config").fetch_one(pool).await
        .context("Erro ao buscar o email de envio")?;

    let nome_provedor = query!("SELECT nome FROM provedor").fetch_one(pool).await
        .context("Erro ao buscar o nome do provedor")?;

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
    context.insert("nome", nome);
    context.insert("valor", &valor.to_string());
    context.insert("data",&data );
    context.insert("periodo", &periodo);
    context.insert("nome_provedor",&nome_provedor.nome);

    let body = TEMPLATES.render("nf_email.html", &context).context("Erro ao renderizar corpo do email")?;
    //TODO parse html template with TERA, put a smartcom logo
    let email = Message::builder()
        .from(from_mail.email.parse().expect("Failed to parse sender email"))
        .to(to.parse().expect("Failed to parse cliente email"))
        .subject("Envio de Nota Fiscal - Serviço Prestado")
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