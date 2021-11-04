use rest_client::{Client, EmptyResponse, Request};
use std::borrow::Cow;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct EmptyHello;

impl Request for EmptyHello {
    type Body = ();
    type Response = EmptyResponse;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }
}

#[tokio::test]
async fn basic_auth() {
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri).basic_auth("user".into(), "pass".into());

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("Authorization", "Basic dXNlcjpwYXNz")) // user:pass in base64
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(&EmptyHello).await.unwrap();
}
