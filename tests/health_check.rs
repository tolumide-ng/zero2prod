#[actix_rt::test]
async fn health_check_works() {
    // arrange
    spawn_app();
    let client = reqwest::Client::new();


    // Act
    let response = client.get("http://127.0.0.1/8000/health-_check")
        .send().await.expect("Failed to execute request");

    assert!(response.status().is_success());
    println!("the response itslef {:#?}", &response);
    assert_eq!(Some(27), response.content_length());
}


async fn spawn_app() {
    let server = zero2prod::run("127.0.0.1:0").expect("Failed to bind address");

    let _ = tokio::spawn(server);
}