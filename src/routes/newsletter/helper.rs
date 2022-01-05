use crate::domain::subscriber_email::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}


#[derive(serde::Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content
}

pub struct ConfirmedSubscriber {
    pub email: SubscriberEmail,
}

