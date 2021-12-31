use tracing;
use crate::email::email_client::EmailClient;
use crate::domain::new_subscriber::NewSubscriber;

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient, 
    new_subscriber: NewSubscriber
) -> Result<(), reqwest::Error> {
    let confirmation_link = "http://my-api.com/subscriptions/confirm";
    let html_body =     &format!("Welcome to our newsletter!<br />`
        Click <a href=\"{}\">here</a> to confirm your subscription.", confirmation_link);
    let plain_body = &format!("Welcome to our newsletter!\nVisit {} to confirm your subscription.", confirmation_link);

    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body).await
}