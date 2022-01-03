use actix_web::{HttpResponse, ResponseError, web};
use sqlx::PgPool;
use actix_web::http::StatusCode;

use crate::errors::helper::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}


#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content
}

struct ConfirmedSubscriber {
    email: String,
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

#[tracing::instrument(
    name = "Get confirmed subscribers", skip(pool)
)]
async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email FROM subscriptions WHERE status = 'confirmed'
        "#,
    ).fetch_all(pool)
        .await?;

    Ok(rows)
}

pub async fn publish_newsletter(
    _body: web::Json<BodyData>, 
    pool: web::Data<PgPool>
) -> Result<HttpResponse, PublishError> {
    let _subsribers = get_confirmed_subscribers(&pool).await?;
    Ok(HttpResponse::Ok().finish())
}
