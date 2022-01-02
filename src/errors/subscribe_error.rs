use actix_web::{http::StatusCode, ResponseError};

use crate::errors::helper::error_chain_fmt;


#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    // #[error(transparent)]
    // UnexpectedError(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}


impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}


impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            // SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}