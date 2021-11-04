use crate::error::{Error, Result};
use crate::pagination::{PaginatedRequest, PaginationState, PaginationType};
use crate::request::{Request, RequestBuilderExt};
use futures::prelude::*;
use reqwest::Client as ReqwestClient;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Clone)]
pub enum Authentication<T, U> {
    Bearer(T),
    Basic(T, T),
    Url { key: U, value: U },
}

/// The main client used for making requests.
///
/// `Client` stores an async Reqwest client as well as the associated
/// base url for the REST server.
#[derive(Clone)]
pub struct Client<'a, T, U> {
    inner: Arc<ReqwestClient>,
    base_url: Cow<'a, str>,
    auth: Option<Authentication<T, U>>,
}

impl<'a, T: Display, U: Serialize> Client<'a, T, U> {
    /// Create a new `Client`.
    pub fn new(base_url: &'a str) -> Self {
        let inner = Arc::new(ReqwestClient::new());

        Self {
            inner,
            base_url: Cow::Borrowed(base_url),
            auth: None,
        }
    }

    pub fn bearer_auth(mut self, token: T) -> Self {
        self.auth = Some(Authentication::Bearer(token));
        self
    }

    pub fn basic_auth(mut self, user: T, pass: T) -> Self {
        self.auth = Some(Authentication::Basic(user, pass));
        self
    }

    pub fn query_auth(mut self, key: U, value: U) -> Self {
        self.auth = Some(Authentication::Url { key, value });
        self
    }

    fn format_request<R: Request>(&'a self, request: &R) -> Result<reqwest::Request> {
        let endpoint = request.endpoint();
        let endpoint = endpoint.trim_matches('/');
        let url = format!("{}/{}", self.base_url, endpoint);

        let req = self
            .inner
            .request(R::METHOD, &url)
            .headers(request.headers())
            .request_body(request.body());

        let req = match &self.auth {
            None => req,
            Some(Authentication::Bearer(token)) => req.bearer_auth(token),
            Some(Authentication::Basic(user, pass)) => req.basic_auth(user, Some(pass)),
            Some(Authentication::Url { key, value }) => req.query(&[(key, value)]),
        };
        req.build().map_err(From::from)
    }

    fn send_raw<R>(&self, req: reqwest::Request) -> impl Future<Output = Result<R>> + 'a
    where
        R: 'a + for<'de> serde::Deserialize<'de>,
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

    /// Send a `Request`
    pub fn send<R: Request>(&'a self, request: &R) -> impl Future<Output = Result<R::Response>> + 'a
    where
        R::Response: 'a,
    {
        let req = self.format_request(request);
        if let Err(e) = req {
            return future::Either::Left(future::ready(Err(e)));
        };
        let req = req.unwrap();

        future::Either::Right(self.send_raw(req))
    }

    pub fn send_all<I, R>(&'a self, requests: I) -> impl Stream<Item = Result<R::Response>> + 'a
    where
        I: Iterator<Item = &'a R> + 'a,
        R: Request + 'a,
    {
        stream::iter(requests)
            .map(move |r| self.send(r).map_into())
            .filter_map(|x| x)
    }

    pub fn send_paginated<R: PaginatedRequest + 'a>(
        &'a self,
        request: &'a R,
    ) -> impl Stream<Item = Result<R::Response>> + 'a {
        stream::try_unfold(
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
        )
    }
}
