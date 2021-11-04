use futures::StreamExt;
use rest_client::Client;
use rest_client::{PaginatedRequest, Paginator, QueryPaginator, Request, RequestBody};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use stream_flatten_iters::TryStreamExt;

mod helpers;
use helpers::get_next_url;

#[derive(Serialize)]
struct GetPassengers {
    size: usize,
}

#[derive(Deserialize, Debug)]
struct Passenger {
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassengersWrapper {
    total_passengers: usize,
    total_pages: usize,
    data: Vec<Passenger>,
}

impl Request for GetPassengers {
    type Body = Self;
    type Response = PassengersWrapper;

    fn endpoint(&self) -> Cow<str> {
        "/v1/passenger".into()
    }

    fn body(&self) -> RequestBody<&Self> {
        RequestBody::Query(self)
    }
}

impl PaginatedRequest for GetPassengers {
    fn paginator(&self) -> Box<dyn Paginator<Self::Response>> {
        Box::new(QueryPaginator::new(get_next_url))
    }
}

#[tokio::main]
pub async fn main() {
    let client: Client<'_, String, String> = Client::new("https://api.instantwebtools.net");
    let req = GetPassengers { size: 100 };

    // Can send request individually
    println!("{:?}", client.send(&req).await);

    // Can send paginated request, returning stream of results
    client
        .send_paginated(&req)
        .map(|maybe_wrapper| maybe_wrapper.map(|wrapper| wrapper.data))
        .try_flatten_iters()
        .for_each(|res| async move { println!("{:?}", res) })
        .await;
}
