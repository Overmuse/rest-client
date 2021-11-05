use crate::utils::{NameGreeting, QueryHello};
use rest_client::Client;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, Request as MockRequest, ResponseTemplate};

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
