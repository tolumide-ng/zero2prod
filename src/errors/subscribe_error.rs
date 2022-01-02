use actix_web::{http::StatusCode, ResponseError};

use crate::errors::store_token_error::StoreTokenError;
use crate::errors::helper::error_chain_fmt;


#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    // DatabaseError(sqlx::Error),
    #[error("Failed to acquire a Postgres connection from the pool")]
    PoolError(sqlx::Error),
    #[error("Failed to insert new subscriber in the database.")]
    InsertSubscriberError(sqlx::Error),
    #[error("Failed to commit SQL transactions to stor a new subscriber.")]
    TransactionCommitError(sqlx::Error),
    #[error("Failed to send a confirmation email")]
    StoreTokenError(StoreTokenError),
    #[error("Failed to send a confirmation email")]
    SendEmailError(reqwest::Error)
}


// impl From<reqwest::Error> for SubscribeError {
//     fn from(e: reqwest::Error) -> Self {
//         Self::SendEmailError(e)
//     }
// }

// impl From<String> for SubscribeError {
//     fn from(e: String) -> Self {
//         Self::ValidationError(e)
//     }
// }

// impl From<StoreTokenError> for SubscribeError {
//     fn from(e: StoreTokenError) -> Self {
//         Self::StoreTokenError(e)
//     }
// }

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}


impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::TransactionCommitError(_)
            | SubscribeError::PoolError(_)
            | SubscribeError::InsertSubscriberError(_)
            | SubscribeError::StoreTokenError(_)
            | SubscribeError::SendEmailError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}


// impl std::fmt::Display for SubscribeError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             SubscribeError::ValidationError(e)  => write!(f, "{}", e),
//             SubscribeError::StoreTokenError(_) => {
//                 write!(f, "Failed to store the confirmation token for a new subscriber.")
//             },
//             SubscribeError::SendEmailError(_) => {
//                 write!(f, "Failed to send a confirmation email.")
//             },
//             SubscribeError::InsertSubscriberError(_) => {
//                 write!(f, "Failed to insert new subscriber in the database")
//             }
//             SubscribeError::TransactionCommitError(_) => {
//                 write!(f, "Faled to commit SQL transaction to store a new subscriber")
//             },
//             SubscribeError::PoolError(_) => {
//                 write!(f, "Failed to acquire a Postgres connection from the pool")
//             }

//         }
//     }
// }

// impl std::error::Error for SubscribeError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             SubscribeError::ValidationError(_) => None,
//             SubscribeError::PoolError(e) => Some(e),
//             SubscribeError::TransactionCommitError(e) => Some(e),
//             SubscribeError::InsertSubscriberError(e) => Some(e),
//             SubscribeError::StoreTokenError(e) => Some(e),
//             SubscribeError::SendEmailError(e) => Some(e),
//         }
//     }
// }

// impl ResponseError for SubscribeError {}