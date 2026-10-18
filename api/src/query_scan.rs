use rocket::request::{FromRequest, Outcome, Request};

/// Raw-query allowlist scan. Rocket's query parsing is always lenient
/// about unknown keys, so parameter enumeration happens here: any key
/// outside `ALLOWED` is collected and the handler answers 400.
pub struct QueryScan {
    pub unknown: Vec<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for QueryScan {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        const ALLOWED: [&str; 5] = ["limit", "offset", "sort", "order", "search"];
        let unknown = request
            .uri()
            .query()
            .map(|query| {
                query
                    .segments()
                    .map(|(name, _)| name.to_string())
                    .filter(|name| !ALLOWED.contains(&name.as_str()))
                    .collect()
            })
            .unwrap_or_default();
        Outcome::Success(QueryScan { unknown })
    }
}

impl QueryScan {
    pub fn is_clean(&self) -> bool {
        self.unknown.is_empty()
    }
}
