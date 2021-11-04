use rest_client::{Client, EmptyResponse, Request};
use std::borrow::Cow;
use wiremock::matchers::{method, path};
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
async fn empty_response() {
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(&EmptyHello).await.unwrap();
}
