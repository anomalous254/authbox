use async_trait::async_trait;

use authbox_core::traits::EmailProvider;

use lettre::{
    AsyncSmtpTransport,
    AsyncTransport,
    Message,
    Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};


#[derive(Debug)]
pub enum SmtpEmailError {
    Message(lettre::error::Error),
    Transport(lettre::transport::smtp::Error),
}


#[derive(Clone)]
pub struct SmtpEmailProvider {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}


impl SmtpEmailProvider {

    pub fn new(
        host: &str,
        username: &str,
        password: &str,
        from: &str,
    ) -> Self {

        let credentials = Credentials::new(
            username.to_owned(),
            password.to_owned(),
        );


        let mailer =
            AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .unwrap()
                .credentials(credentials)
                .build();


        Self {
            mailer,
            from: from.parse().unwrap(),
        }
    }
}



#[async_trait]
impl EmailProvider for SmtpEmailProvider {

    type Error = SmtpEmailError;


    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Self::Error> {


        let email = Message::builder()
            .from(self.from.clone())
            .to(
                to.parse::<Mailbox>()
                    .unwrap()
            )
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_owned())
            .map_err(SmtpEmailError::Message)?;


        self.mailer
            .send(email)
            .await
            .map_err(SmtpEmailError::Transport)?;


        Ok(())
    }
}
