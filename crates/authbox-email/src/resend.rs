use async_trait::async_trait;
use authbox_core::traits::EmailProvider;
use reqwest::Client;
use serde_json::json;


#[derive(Clone)]
pub struct ResendEmailProvider {
    api_key: String,
    from: String,
    client: Client,
}


impl ResendEmailProvider {

    pub fn new<T: Into<String>>(
        api_key: T,
        from: T,
    ) -> Self {

        Self {
            api_key: api_key.into(),
            from: from.into(),
            client: Client::new(),
        }
    }
}



#[async_trait]
impl EmailProvider for ResendEmailProvider {

    type Error = reqwest::Error;


    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Self::Error> {


        self.client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .json(&json!({
                "from": self.from,
                "to": [to],
                "subject": subject,
                "html": body
            }))
            .send()
            .await?
            .error_for_status()?;


        Ok(())
    }
}
