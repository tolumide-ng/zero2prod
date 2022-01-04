use actix_web::{ResponseError};
use actix_web::http::StatusCode;
use crate::domain::subscriber_email::SubscriberEmail;

use crate::errors::helper::error_chain_fmt;


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

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}


impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}