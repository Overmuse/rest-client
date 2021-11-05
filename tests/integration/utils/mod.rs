use rest_client::{EmptyResponse, Request, RequestData};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub mod matchers;

pub struct EmptyHello;

impl Request for EmptyHello {
    type Data = ();
    type Response = EmptyResponse;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }
}

#[derive(Serialize)]
pub struct QueryHello {
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct NameGreeting {
    pub message: String,
}

impl Request for QueryHello {
    type Data = Self;
    type Response = NameGreeting;

    fn endpoint(&self) -> Cow<str> {
        "/hello".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(&self)
    }
}
