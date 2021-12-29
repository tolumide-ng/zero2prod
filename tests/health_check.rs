mod common;

#[actix_rt::test]
async fn health_check_works() {
    // arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();

    let response = client.get(&format!("{}/health_check", &app.address)).send().await.expect("Failed to execuet request");

    // Act
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}



#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20example&email=name%40example.com";

    // Act
    let response = client.post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());


    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "name@example.com");
    assert_eq!(saved.name, "le example");
}



#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=sample", "missing the email"),
        ("email=sample%40email.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        // Act 
        let response = client.post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(400, response.status().as_u16(), 
        // Additional customised error message on test failure
        "The API did not fail with 400 Bad Request when the payload was {}.", error_message
    )
    }
}


// #[actix_rt::test]
// async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
//     let app = common::spawn_app().await;
//     let client = reqwest::Client::new();
//     let test_cases = vec![
//         ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
//         ("name=Ursula&email=", "empty email"),
//         ("name=Ursula&email=definitely-not-an-email", "invalid email"),
//     ];

//     for (body, description) in test_cases {
//         let response = client.post(&format!("{}/subscriptions", &app.address))
//                 .header("Content-Type", "application/x-www-form-urlencoded")
//                 .body(body).send()
//                 .await.expect("Failed to execute request");

//         assert_eq!(400, response.status().as_u16(), "The API did not return a 400 Bad Request when the payload was {}.", description);
//     }
// }
