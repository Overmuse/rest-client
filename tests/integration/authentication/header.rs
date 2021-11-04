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
async fn header_auth() {
    let server = MockServer::start().await;
    let uri = server.uri();
    let auth = vec![("key", "k"), ("secret", "s")];
    let client = Client::new(&uri).header_auth(auth);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header("key", "k"))
        .and(header("secret", "s"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.send(&EmptyHello).await.unwrap();
}
