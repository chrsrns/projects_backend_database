use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, channel};
use std::thread::{self, JoinHandle};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

#[derive(Debug, Clone)]
pub struct CapturedHttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

fn read_request_from_stream(stream: &mut TcpStream) -> CapturedHttpRequest {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];

    let header_end = loop {
        let n = stream.read(&mut tmp).expect("read http request");
        if n == 0 {
            panic!("unexpected EOF before end of HTTP headers");
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(end) = find_header_end(&buf) {
            break end;
        }
    };

    let head = std::str::from_utf8(&buf[..header_end]).expect("ascii http headers");
    let mut lines = head.split("\r\n");
    let request_line = lines.next().expect("request line");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.push((
                key.trim().to_lowercase().to_string(),
                value.trim().to_string(),
            ));
        }
    }

    let content_length = headers
        .iter()
        .find(|(k, _)| k == "content-length")
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);

    let body_start = header_end;
    let mut body = Vec::new();
    if content_length > 0 {
        let body_in_buf = if buf.len() >= body_start + content_length {
            content_length
        } else {
            buf.len() - body_start
        };
        body.extend_from_slice(&buf[body_start..body_start + body_in_buf]);

        while body.len() < content_length {
            let n = stream.read(&mut tmp).expect("read http body");
            if n == 0 {
                break;
            }
            let remaining = content_length - body.len();
            let take = n.min(remaining);
            body.extend_from_slice(&tmp[..take]);
        }
    }

    CapturedHttpRequest {
        method,
        path,
        version,
        headers,
        body,
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        303 => "See Other",
        404 => "Not Found",
        502 => "Bad Gateway",
        _ => "OK",
    }
}

pub struct MockUpstreamHttp {
    pub port: u16,
    request_rx: Receiver<CapturedHttpRequest>,
    handle: Option<JoinHandle<()>>,
}

impl MockUpstreamHttp {
    pub fn with_response(
        status: u16,
        content_type: Option<&str>,
        location: Option<&str>,
        body: &[u8],
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock upstream");
        let port = listener.local_addr().unwrap().port();

        let (ready_tx, ready_rx) = channel();
        let (request_tx, request_rx) = channel();

        let response_body = body.to_vec();
        let content_type = content_type.map(|s| s.to_string());
        let location = location.map(|s| s.to_string());

        let handle = thread::spawn(move || {
            ready_tx.send(()).ok();
            let (mut stream, _) = listener.accept().expect("accept mock connection");
            let request = read_request_from_stream(&mut stream);

            let mut response = format!(
                "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n",
                status,
                reason_phrase(status),
                response_body.len()
            );
            if let Some(ct) = content_type {
                response.push_str(&format!("Content-Type: {}\r\n", ct));
            }
            if let Some(loc) = location {
                response.push_str(&format!("Location: {}\r\n", loc));
            }
            response.push_str("\r\n");

            stream
                .write_all(response.as_bytes())
                .expect("write response headers");
            stream
                .write_all(&response_body)
                .expect("write response body");
            stream.flush().expect("flush response");

            request_tx.send(request).ok();
        });

        ready_rx.recv().expect("mock upstream ready");

        Self {
            port,
            request_rx,
            handle: Some(handle),
        }
    }

    pub fn captured_request(self) -> CapturedHttpRequest {
        let request = self.request_rx.recv().expect("receive captured request");
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
        request
    }
}

#[derive(Debug, Clone)]
pub struct CapturedWsRequest {
    pub path: String,
    pub protocol: Option<String>,
}

pub struct MockUpstreamWs {
    pub port: u16,
    path_rx: Receiver<String>,
    protocol_rx: Receiver<Option<String>>,
    handle: Option<JoinHandle<()>>,
}

impl MockUpstreamWs {
    pub fn start() -> Self {
        let (port_tx, port_rx) = channel();
        let (path_tx, path_rx) = channel();
        let (protocol_tx, protocol_rx) = channel();

        let handle = thread::spawn(move || {
            let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime.block_on(async {
                let listener = rocket::tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind ws mock");
                let port = listener.local_addr().unwrap().port();
                port_tx.send(port).expect("send ws mock port");

                let (stream, _) = listener.accept().await.expect("accept ws connection");

                let path_tx = path_tx.clone();
                let protocol_tx = protocol_tx.clone();

                let callback = move |req: &Request, mut response: Response| {
                    let path = req.uri().path().to_string();
                    let protocol = req
                        .headers()
                        .get("Sec-WebSocket-Protocol")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    if let Some(ref proto) = protocol {
                        if let Ok(value) = proto.parse() {
                            response
                                .headers_mut()
                                .append("Sec-WebSocket-Protocol", value);
                        }
                    }

                    let _ = path_tx.send(path);
                    let _ = protocol_tx.send(protocol);
                    Ok(response)
                };

                let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback)
                    .await
                    .expect("accept ws handshake");

                let (mut write, mut read) = ws_stream.split();

                while let Some(result) = read.next().await {
                    let msg = match result {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };

                    let forward = match msg {
                        Message::Text(t) => Message::Text(t),
                        Message::Binary(b) => Message::Binary(b),
                        Message::Ping(_) | Message::Pong(_) => continue,
                        Message::Close(_) => {
                            let _ = write.send(Message::Close(None)).await;
                            break;
                        }
                        _ => break,
                    };

                    if write.send(forward).await.is_err() {
                        break;
                    }
                }
            });
        });

        let port = port_rx.recv().expect("receive ws mock port");

        Self {
            port,
            path_rx,
            protocol_rx,
            handle: Some(handle),
        }
    }

    pub fn captured_request(&self) -> CapturedWsRequest {
        let path = self.path_rx.recv().expect("receive captured ws path");
        let protocol = self
            .protocol_rx
            .recv()
            .expect("receive captured ws protocol");
        CapturedWsRequest { path, protocol }
    }

    pub fn captured_protocol(self) -> Option<String> {
        let _ = self.path_rx.recv().expect("receive captured ws path");
        self.protocol_rx
            .recv()
            .expect("receive captured ws protocol")
    }
}
