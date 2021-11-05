use crate::utils::EmptyHello;
use rest_client::Client;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn basic_auth() {
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri).basic_auth("user", "pass");

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("Authorization", "Basic dXNlcjpwYXNz")) // user:pass in base64
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(&EmptyHello).await.unwrap();
}
