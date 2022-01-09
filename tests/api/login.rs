use std::collections::HashSet;

use crate::helpers::app::spawn_app;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;

    let flash_cookies = response.cookies().find(|c| c.name() == "_flash").unwrap();

    assert_eq!(flash_cookies.valu(), "Authentication failed");

    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), "/login");
}

