use rocket::Request;
use rocket::http::{Method, Status};
use rocket::response::Redirect;
use rocket::route::{Handler, Outcome, Route};

#[derive(Clone)]
pub struct FrontendTrailingSlashRedirectHandler;

#[rocket::async_trait]
impl Handler for FrontendTrailingSlashRedirectHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: rocket::Data<'r>) -> Outcome<'r> {
        if should_redirect_frontend_trailing_slash(req.uri().path().as_str()) {
            return Outcome::from(req, Redirect::to(normalized_frontend_redirect_target(req)));
        }

        Outcome::forward(data, Status::NotFound)
    }
}

fn should_redirect_frontend_trailing_slash(path: &str) -> bool {
    path.starts_with("/resume_editor/") && path.ends_with('/') && path.len() > 1
}

fn normalized_frontend_redirect_target(req: &Request<'_>) -> String {
    let request_uri = req.uri().to_string();

    match request_uri.split_once('?') {
        Some((path, query)) => format!("{}?{}", path.trim_end_matches('/'), query),
        None => request_uri.trim_end_matches('/').to_string(),
    }
}

pub fn frontend_trailing_slash_redirect_routes() -> Vec<Route> {
    vec![Route::ranked(
        99,
        Method::Get,
        "/<path..>",
        FrontendTrailingSlashRedirectHandler,
    )]
}
