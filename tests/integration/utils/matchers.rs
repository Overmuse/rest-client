use wiremock::{Match, Request};

pub struct MissingQuery<'a>(&'a str);

impl<'a> MissingQuery<'a> {
    pub fn new(query: &'a str) -> Self {
        Self(query)
    }
}

impl Match for MissingQuery<'_> {
    fn matches(&self, request: &Request) -> bool {
        request
            .url
            .query_pairs()
            .find(|(k, _)| k == self.0)
            .is_none()
    }
}
