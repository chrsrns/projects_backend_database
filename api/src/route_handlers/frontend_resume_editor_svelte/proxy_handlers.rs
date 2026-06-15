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

pub struct Subprotocol(Option<String>);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for Subprotocol {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let proto = request
            .headers()
            .get_one("Sec-WebSocket-Protocol")
            .map(String::from);
        rocket::request::Outcome::Success(Subprotocol(proto))
    }
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

#[cfg(feature = "frontend_resume_editor_svelte")]
#[get("/resume_editor/<path..>?<query..>", rank = 100)]
pub async fn frontend_websocket_proxy_handler(
    path: FrontendProxyPath,
    query: Option<std::collections::HashMap<String, String>>,
    node_cfg: &rocket::State<NodeConfig>,
    subprotocol: Subprotocol,
    ws: ws::WebSocket,
) -> ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

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

    let upstream_url = format!("ws://localhost:{}{}", node_cfg.port, full_path);

    // Capture the subprotocol header from the original request so we can forward it to Vite
    let subprotocol = subprotocol.0;

    log::info!(
        "websocket proxy step=prepare_request path={} upstream_url={} subprotocol={:?}",
        full_path,
        upstream_url,
        subprotocol
    );

    ws.channel(move |mut client_stream| {
        Box::pin(async move {
            let mut upstream_req = match upstream_url.clone().into_client_request() {
                Ok(req) => req,
                Err(err) => {
                    log::error!(
                        "websocket proxy step=create_request path={} upstream_url={} error={}",
                        full_path,
                        upstream_url,
                        err
                    );
                    let _ = client_stream.send(ws::Message::Close(None)).await;
                    return Ok(());
                }
            };

            // Forward the Sec-WebSocket-Protocol header required by Vite HMR
            if let Some(proto) = &subprotocol {
                if let Ok(header_value) = proto.parse() {
                    upstream_req
                        .headers_mut()
                        .insert("Sec-WebSocket-Protocol", header_value);
                }
            }

            // Connect to upstream WebSocket server
            let (mut upstream_stream, _) = match tokio_tungstenite::connect_async(upstream_req)
                .await
            {
                Ok(result) => result,
                Err(err) => {
                    log::error!(
                        "websocket proxy step=connect_upstream path={} upstream_url={} error={}",
                        full_path,
                        upstream_url,
                        err
                    );
                    let _ = client_stream.send(ws::Message::Close(None)).await;
                    return Ok(());
                }
            };

            log::info!(
                "websocket proxy step=connected path={} upstream_url={}",
                full_path,
                upstream_url
            );

            // Bidirectional proxying between client and upstream
            loop {
                rocket::tokio::select! {
                    // Client -> Upstream
                    client_msg = client_stream.next() => {
                        match client_msg {
                            Some(Ok(msg)) => {
                                let converted_msg = convert_rocket_to_tungstenite(msg);
                                if upstream_stream.send(converted_msg).await.is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) | None => break,
                        }
                    }

                    // Upstream -> Client
                    upstream_msg = upstream_stream.next() => {
                        match upstream_msg {
                            Some(Ok(msg)) => {
                                let converted_msg = convert_tungstenite_to_rocket(msg);
                                if client_stream.send(converted_msg).await.is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) | None => break,
                        }
                    }
                }
            }

            Ok(())
        })
    })
}

#[cfg(feature = "frontend_resume_editor_svelte")]
fn convert_rocket_to_tungstenite(msg: ws::Message) -> tokio_tungstenite::tungstenite::Message {
    match msg {
        ws::Message::Text(text) => tokio_tungstenite::tungstenite::Message::Text(text.into()),
        ws::Message::Binary(data) => tokio_tungstenite::tungstenite::Message::Binary(data.into()),
        ws::Message::Close(reason) => {
            tokio_tungstenite::tungstenite::Message::Close(reason.map(|r| {
                tokio_tungstenite::tungstenite::protocol::CloseFrame {
                    code: u16::from(r.code).into(),
                    reason: r.reason.to_string().into(),
                }
            }))
        }
        ws::Message::Ping(data) => tokio_tungstenite::tungstenite::Message::Ping(data.into()),
        ws::Message::Pong(data) => tokio_tungstenite::tungstenite::Message::Pong(data.into()),
        ws::Message::Frame(_) => tokio_tungstenite::tungstenite::Message::Close(None),
    }
}

#[cfg(feature = "frontend_resume_editor_svelte")]
fn convert_tungstenite_to_rocket(msg: tokio_tungstenite::tungstenite::Message) -> ws::Message {
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => ws::Message::Text(text.to_string()),
        tokio_tungstenite::tungstenite::Message::Binary(data) => ws::Message::Binary(data.to_vec()),
        tokio_tungstenite::tungstenite::Message::Close(reason) => {
            ws::Message::Close(reason.map(|r| ws::frame::CloseFrame {
                code: u16::from(r.code).into(),
                reason: r.reason.to_string().into(),
            }))
        }
        tokio_tungstenite::tungstenite::Message::Ping(data) => ws::Message::Ping(data.to_vec()),
        tokio_tungstenite::tungstenite::Message::Pong(data) => ws::Message::Pong(data.to_vec()),
        _ => ws::Message::Close(None),
    }
}
