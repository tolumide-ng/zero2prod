use actix_web::ResponseError;

#[derive(Debug)]
pub struct StoreTokenError(sqlx::Error);


impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to stroe a subscription token"
        )
    }
}


impl ResponseError for StoreTokenError {}
