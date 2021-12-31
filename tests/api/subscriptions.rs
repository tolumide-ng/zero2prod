use crate::helpers::{spawn_app};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};


#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20example&email=name%40example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscription(body.to_string()).await;

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
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=sample", "missing the email"),
        ("email=sample%40email.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        // Act 

        let response = app.post_subscription(invalid_body.to_string()).await;

        // Assert
        assert_eq!(400, response.status().as_u16(), 
        // Additional customised error message on test failure
        "The API did not fail with 400 Bad Request when the payload was {}.", error_message
    )
    }
}


#[actix_rt::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscription(body.to_string()).await;

        assert_eq!(400, response.status().as_u16(), "The API did not return a 400 Bad Request when the payload was {}.", description);
    }
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscription(body.into()).await;
}