use crate::utils::{NameGreeting, QueryHello};
use futures::StreamExt;
use rest_client::Client;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request as MockRequest, ResponseTemplate};

#[tokio::test]
async fn query_multiple() {
    let server = MockServer::start().await;
    let uri = server.uri();
    let client = Client::new(&uri);

    Mock::given(method("GET"))
        .and(path("/hello"))
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

    let reqs = &[
        QueryHello {
            name: "world".into(),
        },
        QueryHello {
            name: "again".into(),
        },
    ];

    let mut response = client.send_all(reqs);
    assert_eq!(
        response.next().await.unwrap().unwrap(),
        NameGreeting {
            message: "Hello, world!".into(),
        }
    );
    assert_eq!(
        response.next().await.unwrap().unwrap(),
        NameGreeting {
            message: "Hello, again!".into(),
        }
    );
    assert!(response.next().await.is_none());
}
