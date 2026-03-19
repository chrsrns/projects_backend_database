use std::io::Cursor;

use reqwest::redirect::Policy;
use rocket::{
    Request,
    http::{
        Status,
        uri::{Segments, fmt::Path},
    },
    request::FromSegments,
    response::{Responder, Response},
};
use shared::node_config::NodeConfig;

pub struct ProxyResponse {
    status: Status,
    content_type: Option<String>,
    location: Option<String>,
    body: Vec<u8>,
}

pub struct FrontendProxyPath(String);

impl<'r> FromSegments<'r> for FrontendProxyPath {
    type Error = std::convert::Infallible;

    fn from_segments(segments: Segments<'r, Path>) -> Result<Self, Self::Error> {
        let segments_vec: Vec<_> = segments.collect();

        Ok(Self(segments_vec.join("/")))
    }
}

impl<'r> Responder<'r, 'static> for ProxyResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut builder = Response::build();
        builder.status(self.status);
        if let Some(content_type) = self.content_type {
            builder.raw_header("Content-Type", content_type);
        }
        if let Some(location) = self.location {
            builder.raw_header("Location", location);
        }

        builder
            .sized_body(self.body.len(), Cursor::new(self.body))
            .ok()
    }
}

async fn proxy_frontend_path(path: &str, node_port: u16) -> Result<ProxyResponse, Status> {
    let upstream_url = format!("http://localhost:{}{path}", node_port);
    log::info!(
        "frontend proxy step=prepare_request path={} upstream_url={}",
        path,
        upstream_url
    );

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .map_err(|err| {
            log::error!(
                "frontend proxy step=build_client path={} upstream_url={} error={}",
                path,
                upstream_url,
                err
            );
            Status::InternalServerError
        })?;

    let upstream_response = client.get(&upstream_url).send().await.map_err(|err| {
        log::error!(
            "frontend proxy step=send_request path={} upstream_url={} error={}",
            path,
            upstream_url,
            err
        );
        Status::BadGateway
    })?;

    log::info!(
        "frontend proxy step=received_response path={} upstream_url={} upstream_status={}",
        path,
        upstream_url,
        upstream_response.status()
    );

    let status =
        Status::from_code(upstream_response.status().as_u16()).unwrap_or(Status::BadGateway);
    log::info!(
        "frontend proxy step=map_status path={} upstream_status={} rocket_status={}",
        path,
        upstream_response.status(),
        status
    );

    let content_type = upstream_response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let location = upstream_response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    log::info!(
        "frontend proxy step=read_headers path={} content_type={} location={}",
        path,
        content_type.as_deref().unwrap_or("<none>"),
        location.as_deref().unwrap_or("<none>")
    );

    let body = upstream_response.bytes().await.map_err(|err| {
        log::error!(
            "frontend proxy step=read_body path={} upstream_url={} error={}",
            path,
            upstream_url,
            err
        );
        Status::BadGateway
    })?;

    log::info!(
        "frontend proxy step=complete path={} status={} body_bytes={}",
        path,
        status,
        body.len()
    );

    Ok(ProxyResponse {
        status,
        content_type,
        location,
        body: body.to_vec(),
    })
}

#[get("/", rank = 100)]
pub async fn frontend_index_proxy_handler(
    node_cfg: &rocket::State<NodeConfig>,
) -> Result<ProxyResponse, Status> {
    proxy_frontend_path("/", node_cfg.port).await
}

#[get("/resume_editor/<path..>?<query..>", rank = 101)]
pub async fn frontend_proxy_handler(
    path: FrontendProxyPath,
    query: Option<std::collections::HashMap<String, String>>,
    node_cfg: &rocket::State<NodeConfig>,
) -> Result<ProxyResponse, Status> {
    let normalized_path = path.0;

    let full_path = match query {
        Some(q) if !q.is_empty() => {
            let query_string: String = q
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("/resume_editor/{}?{}", normalized_path, query_string)
        }
        _ => format!("/resume_editor/{}", normalized_path),
    };

    proxy_frontend_path(&full_path, node_cfg.port).await
}
