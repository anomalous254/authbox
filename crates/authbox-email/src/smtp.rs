use async_trait::async_trait;
use authbox_core::traits::EmailProvider;
use lettre::{
    AsyncTransport,
    AsyncSmtpTransport,
    Tokio1Executor,
    Message,
    transport::smtp::authentication::Credentials,
};



#[derive(Clone)]
pub struct SmtpEmailProvider {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}



impl SmtpEmailProvider {
    pub fn new(
        host: &str,
        username: &str,
        password: &str,
        from: &str,
    ) -> Self {
        let credentials =
            Credentials::new(
                username.to_string(),
                password.to_string()
            );

        let mailer =
            AsyncSmtpTransport::<Tokio1Executor>
                ::relay(host)
                .unwrap()
                .credentials(credentials)
                .build();

        Self {
            mailer,
            from: from.to_string(),
        }
    }
}




#[async_trait]
impl EmailProvider for SmtpEmailProvider {
    type Error = lettre::transport::smtp::Error;

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Self::Error> {


        let email =
            Message::builder()
                .from(self.from.parse().unwrap())
                .to(to.parse().unwrap())
                .subject(subject)
                .header(
                    lettre::message::header::ContentType::TEXT_HTML
                )
                .body(body.to_string())
                .unwrap();



        self.mailer
            .send(email)
            .await?;


        Ok(())
    }
}
