use crate::error::{Error, Result};
use crate::pagination::{PaginatedRequest, PaginationState, PaginationType, Paginator};
use crate::request::{Request, RequestBuilderExt};
use futures::prelude::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client as ReqwestClient;
use std::convert::TryFrom;
use std::sync::Arc;

#[derive(Clone)]
enum Authorization {
    Bearer(String),
    Basic(String, String),
    Query(Vec<(String, String)>),
    Header(HeaderMap<HeaderValue>),
}

/// The main client used for making requests.
///
/// `Client` stores an async Reqwest client as well as the associated
/// base url and possible authorization details for the REST server.
#[derive(Clone)]
pub struct Client {
    inner: Arc<ReqwestClient>,
    base_url: String,
    auth: Option<Authorization>,
}

impl Client {
    /// Create a new `Client`.
    pub fn new<S: ToString>(base_url: S) -> Self {
        let inner = Arc::new(ReqwestClient::new());

        Self {
            inner,
            base_url: base_url.to_string(),
            auth: None,
        }
    }

    /// Enable bearer authentication for the client
    pub fn bearer_auth<S: ToString>(mut self, token: S) -> Self {
        self.auth = Some(Authorization::Bearer(token.to_string()));
        self
    }

    /// Enable basic authentication for the client
    pub fn basic_auth<S: ToString>(mut self, user: S, pass: S) -> Self {
        self.auth = Some(Authorization::Basic(user.to_string(), pass.to_string()));
        self
    }

    /// Enable query authentication for the client
    pub fn query_auth<S: ToString>(mut self, pairs: Vec<(S, S)>) -> Self {
        let pairs = pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.auth = Some(Authorization::Query(pairs));
        self
    }

    /// Enable custom header authentication for the client
    pub fn header_auth<S: ToString>(mut self, pairs: Vec<(S, S)>) -> Self {
        let mut map = HeaderMap::new();
        for (k, v) in pairs {
            let k = k.to_string();
            let v = v.to_string();
            map.insert(
                HeaderName::try_from(&k).expect("Failed to create HeaderName"),
                HeaderValue::from_str(&v).expect("Failed to create HeaderValue"),
            );
        }
        self.auth = Some(Authorization::Header(map));
        self
    }

    fn format_request<R: Request>(&self, request: &R) -> Result<reqwest::Request> {
        let endpoint = request.endpoint();
        let endpoint = endpoint.trim_matches('/');
        let url = format!("{}/{}", self.base_url, endpoint);

        let req = self
            .inner
            .request(R::METHOD, &url)
            .headers(request.headers())
            .request_data(request.data());

        let req = match &self.auth {
            None => req,
            Some(Authorization::Bearer(token)) => req.bearer_auth(token),
            Some(Authorization::Basic(user, pass)) => req.basic_auth(user, Some(pass)),
            Some(Authorization::Query(pairs)) => req.query(&pairs),
            Some(Authorization::Header(pairs)) => req.headers(pairs.clone()),
        };
        req.build().map_err(From::from)
    }

    fn send_raw<R>(&self, req: reqwest::Request) -> impl Future<Output = Result<R>>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        self.inner
            .execute(req)
            .map_err(From::from)
            .and_then(|res| async {
                let status = res.status();
                if status.is_success() {
                    res.json().await.map_err(From::from)
                } else if status.is_client_error() {
                    Err(Error::ClientError(status, res.text().await.unwrap()))
                } else {
                    Err(Error::ServerError(status, res.text().await.unwrap()))
                }
            })
    }

    /// Send a single `Request`
    pub async fn send<R: Request>(&self, request: &R) -> Result<R::Response> {
        let req = self.format_request(request)?;
        self.send_raw(req).await
    }

    /// Send multiple `Request`s, returing a stream of results
    pub fn send_all<'a, I, R>(
        &'a self,
        requests: I,
    ) -> impl Stream<Item = Result<R::Response>> + Unpin + 'a
    where
        I: IntoIterator<Item = &'a R> + 'a,
        R: Request + 'a,
    {
        Box::pin(
            stream::iter(requests.into_iter())
                .map(move |r| self.send(r).map_into())
                .filter_map(|x| x),
        )
    }

    /// Send a paginated request, returning a stream of results
    pub fn send_paginated<'a, R: PaginatedRequest>(
        &'a self,
        request: &'a R,
    ) -> impl Stream<Item = Result<R::Response>> + Unpin + 'a {
        Box::pin(stream::try_unfold(
            (request.paginator(), PaginationState::Start(None)),
            move |(paginator, state)| async move {
                let mut base_request = self.format_request(request)?;
                let page = match state.clone() {
                    PaginationState::Start(page) => page,
                    PaginationState::Next(page) => Some(page),
                    PaginationState::End => return Ok(None),
                };
                if let Some(page) = page.as_ref() {
                    match page {
                        PaginationType::Query(queries) => {
                            let mut existing = base_request.url_mut().query_pairs_mut();
                            for (key, val) in queries.iter() {
                                existing.append_pair(key, val);
                            }
                        }
                    };
                }

                let response = self.send_raw(base_request).await?;
                let state = paginator.next(&state, &response);
                Ok(Some((response, (paginator, state))))
            },
        ))
    }
}
