use reqwest::Client;
use crate::domain::subscriber_email::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
}

impl EmailClient {
    pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), String> {
        todo!()
    }

    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender
        }
    }
}