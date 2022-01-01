use crate::helpers::app::spawn_app;


#[actix_rt::test]
async fn health_check_works() {
    // arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client.get(&format!("{}/health_check", &app.address)).send().await.expect("Failed to execuet request");

    // Act
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
