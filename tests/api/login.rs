use std::collections::HashSet;

use crate::helpers::app::spawn_app;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Part 1 - Try to login
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    
    let response = app.post_login(&login_body).await;
    let flash_cookies = response.cookies().find(|c| c.name() == "_flash").unwrap();
    assert_eq!(flash_cookies.value(), "Authentication failed");
    
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), "/login");
    
    // Part 2 - Follow the redirect
    let response = app.get_login().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
    
    // Part 3 - Reload the login page
    let response = app.get_login().await;
    let html_page = response.text().await.unwrap();
    assert!(!html_page.contains("<p><i>Authentication failed</i></p>"));
}



#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(
        response.headers().get("Location").unwrap(),
        "/admin/dashboard"
    );

    let response = app.get_admin_dashboard().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)))
}
