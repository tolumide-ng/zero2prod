use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

use super::app::TestApp;

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub async fn create_unconfirmed_subcriber(app: &TestApp) {
    let body = "name=le%20guin&email=ursulua_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into())
        .await
        .error_for_status()
        .unwrap();
}