use rest_client::{Client, Request, RequestBody};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, Request as MockRequest, ResponseTemplate};

#[derive(Serialize)]
struct QueryHello {
    name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct NameGreeting {
    message: String,
}

impl Request for QueryHello {
    type Body = Self;
    type Response = NameGreeting;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }

    fn body(&self) -> RequestBody<&Self> {
        RequestBody::Query(&self)
    }
}

#[tokio::test]
async fn query() {
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(query_param("name", "Sebastian"))
        .respond_with(|req: &MockRequest| {
            let name = req
                .url
                .query_pairs()
                .find(|(k, _)| k == "name")
                .map(|(_, v)| v)
                .unwrap();
            let body = NameGreeting {
                message: format!("Hello, {}!", name),
            };
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;

    let response = client
        .send(&QueryHello {
            name: "Sebastian".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        response,
        NameGreeting {
            message: "Hello, Sebastian!".into(),
        }
    );
}
