use reqwest::Url;
use wiremock::{ResponseTemplate, Mock};
use wiremock::matchers::{path, method};

use crate::helpers::app::spawn_app;

#[actix_rt::test]
async fn confirmation_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address)).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}


#[actix_rt::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursual_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Extract the link from on eof the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(&body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    confirmation_link.set_port(Some(app.port)).unwrap();

    println!("THE CONFIRMATION LINK>>>>>>>>>>>>> {}", confirmation_link);

    let response = reqwest::get(confirmation_link)
        .await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}


// #[actix_rt::test]
// async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
//     let app = spawn_app().await;
//     let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

//     Mock::given(path("/email"))
//         .and(method("POST"))
//         .respond_with(ResponseTemplate::new(200))
//         .mount(&app.email_server)
//         .await;

//     app.post_subscription(body.into()).await;
//     let email_request = &app.email_server.received_requests().await.unwrap()[0];
//     let confirmation_links = app.get_confirmation_links(&email_request);
// }