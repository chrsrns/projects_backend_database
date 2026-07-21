use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rocket::http::Status;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::protocol::Role;

mod support;

use support::proxy_mock::{CapturedHttpRequest, MockUpstreamHttp, MockUpstreamWs};

fn rocket_client(upstream_port: u16) -> rocket::local::blocking::Client {
    let rocket = api::build_rocket_with_hub(
        api::realtime::Hub::new(),
        shared::node_config::NodeConfig {
            port: upstream_port,
        },
    );
    rocket::local::blocking::Client::tracked(rocket).expect("valid rocket")
}

fn find_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn spawn_rocket(
    upstream_port: u16,
) -> (
    rocket::tokio::runtime::Runtime,
    rocket::tokio::task::JoinHandle<()>,
    rocket::Shutdown,
    u16,
) {
    let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");
    let (shutdown, server, rocket_port) = runtime.block_on(async {
        let rocket_port = find_free_port();
        let rocket = api::build_rocket_with_hub(
            api::realtime::Hub::new(),
            shared::node_config::NodeConfig {
                port: upstream_port,
            },
        )
        .configure(
            rocket::Config::figment()
                .merge(("address", "127.0.0.1"))
                .merge(("port", rocket_port)),
        );

        let ignited = rocket.ignite().await.expect("ignite rocket");
        let shutdown = ignited.shutdown();
        let server = rocket::tokio::spawn(async move {
            let _ = ignited.launch().await;
        });

        rocket::tokio::time::sleep(Duration::from_millis(150)).await;

        (shutdown, server, rocket_port)
    });

    (runtime, server, shutdown, rocket_port)
}

fn path_and_query(req: &CapturedHttpRequest) -> (&str, Option<&str>) {
    match req.path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (req.path.as_str(), None),
    }
}

async fn raw_ws_connect(
    rocket_port: u16,
    path: &str,
    subprotocol: Option<&str>,
) -> tokio_tungstenite::WebSocketStream<rocket::tokio::net::TcpStream> {
    let mut stream = rocket::tokio::net::TcpStream::connect(("127.0.0.1", rocket_port))
        .await
        .expect("connect tcp");

    let key = generate_key();
    let mut request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Connection: Upgrade\r\n\
         Upgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Key: {}\r\n",
        path, rocket_port, key
    );
    if let Some(proto) = subprotocol {
        request.push_str(&format!("Sec-WebSocket-Protocol: {}\r\n", proto));
    }
    request.push_str("\r\n");

    rocket::tokio::io::AsyncWriteExt::write_all(&mut stream, request.as_bytes())
        .await
        .expect("write http request");

    let mut buf = [0u8; 1024];
    let mut read = 0;
    loop {
        let n = rocket::tokio::io::AsyncReadExt::read(&mut stream, &mut buf[read..])
            .await
            .expect("read http response");
        if n == 0 {
            panic!("unexpected EOF before end of http response");
        }
        read += n;
        if buf[..read].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    tokio_tungstenite::WebSocketStream::from_raw_socket(stream, Role::Client, None).await
}

#[test]
fn test_get_root_index_proxied() {
    let mock = MockUpstreamHttp::with_response(200, Some("text/html"), None, b"<h1>index</h1>");
    let client = rocket_client(mock.port);

    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_bytes().as_deref(),
        Some(b"<h1>index</h1>".as_slice())
    );

    let req = mock.captured_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/");
}

#[test]
fn test_get_resume_editor_path_proxied() {
    let mock =
        MockUpstreamHttp::with_response(200, Some("text/plain"), None, b"resume editor content");
    let client = rocket_client(mock.port);

    let response = client.get("/resume_editor/auth/login").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_bytes().as_deref(),
        Some(b"resume editor content".as_slice())
    );

    let req = mock.captured_request();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/resume_editor/auth/login");
}

#[test]
fn test_query_string_preserved_exactly() {
    let mock = MockUpstreamHttp::with_response(200, Some("text/plain"), None, b"ok");
    let client = rocket_client(mock.port);

    let response = client
        .get("/resume_editor/search?a=1&b=2&a=3&special=%26%3D&z=last")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let req = mock.captured_request();
    assert_eq!(req.method, "GET");
    let (path, query) = path_and_query(&req);
    assert_eq!(path, "/resume_editor/search");
    let query = query.expect("query string present");
    assert_eq!(query, "a=1&b=2&a=3&special=%26%3D&z=last");
}

#[test]
fn test_trailing_slash_redirect() {
    let mock = MockUpstreamHttp::with_response(200, Some("text/plain"), None, b"ok");
    let client = rocket_client(mock.port);

    let response = client.get("/resume_editor/auth/login/").dispatch();

    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(
        response.headers().get_one("Location"),
        Some("/resume_editor/auth/login")
    );

    let response_with_query = client
        .get("/resume_editor/auth/login/?redirect=next&other=1")
        .dispatch();

    assert_eq!(response_with_query.status(), Status::SeeOther);
    assert_eq!(
        response_with_query.headers().get_one("Location"),
        Some("/resume_editor/auth/login?redirect=next&other=1")
    );
}

#[test]
fn test_websocket_upgrade_bypasses_redirect() {
    let mock = MockUpstreamWs::start();
    let (_runtime, server, shutdown, rocket_port) = spawn_rocket(mock.port);

    let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async {
        let ws_url = format!("ws://127.0.0.1:{}/resume_editor/auth/login/", rocket_port);
        let request = ws_url
            .into_client_request()
            .expect("build websocket request");

        let (mut socket, _) = tokio_tungstenite::connect_async(request)
            .await
            .expect("connect websocket");

        let captured = mock.captured_request();
        assert_eq!(captured.path, "/resume_editor/auth/login");

        let _ = socket.send(Message::Close(None)).await;
        let _ = socket.close(None).await;

        shutdown.notify();
        let _ = server.await;
    });
}

#[test]
fn test_websocket_bidirectional_text_binary_ping_pong_close() {
    let mock = MockUpstreamWs::start();
    let (runtime, server, shutdown, rocket_port) = spawn_rocket(mock.port);

    runtime.block_on(async {
        let ws_url = format!("ws://127.0.0.1:{}/resume_editor/socket", rocket_port);
        let (mut socket, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("connect websocket");

        socket
            .send(Message::Text("hello".into()))
            .await
            .expect("send text");
        let text = socket.next().await.unwrap().unwrap();
        assert_eq!(text, Message::Text("hello".into()));

        socket
            .send(Message::Binary(vec![1, 2, 3].into()))
            .await
            .expect("send binary");
        let binary = socket.next().await.unwrap().unwrap();
        assert_eq!(binary, Message::Binary(vec![1, 2, 3].into()));

        socket
            .send(Message::Ping(vec![42].into()))
            .await
            .expect("send ping");
        let pong = socket.next().await.unwrap().unwrap();
        assert_eq!(pong, Message::Pong(vec![42].into()));

        socket.send(Message::Close(None)).await.expect("send close");

        let mut close_received = false;
        while !close_received {
            let next = rocket::tokio::time::timeout(Duration::from_secs(1), socket.next()).await;
            match next {
                Ok(Some(Ok(Message::Close(_)))) => close_received = true,
                Ok(Some(Ok(_))) => continue,
                Ok(None) => break,
                Ok(Some(Err(err))) => panic!("websocket error: {err}"),
                Err(_) => panic!("timeout waiting for close frame"),
            }
        }
        assert!(close_received, "did not receive close frame");

        shutdown.notify();
        let _ = server.await;
    });
}

#[test]
fn test_websocket_subprotocol_forwarded() {
    let mock = MockUpstreamWs::start();
    let (runtime, server, shutdown, rocket_port) = spawn_rocket(mock.port);

    runtime.block_on(async {
        let mut socket = raw_ws_connect(rocket_port, "/resume_editor/hmr", Some("vite")).await;

        let protocol = mock.captured_protocol();
        assert_eq!(protocol.as_deref(), Some("vite"));

        socket
            .send(Message::Text("ping".into()))
            .await
            .expect("send text");
        let echo = socket.next().await.unwrap().unwrap();
        assert_eq!(echo, Message::Text("ping".into()));

        let _ = socket.send(Message::Close(None)).await;
        let _ = socket.close(None).await;

        shutdown.notify();
        let _ = server.await;
    });
}

#[test]
fn test_location_header_rewrite() {
    let relative_mock =
        MockUpstreamHttp::with_response(303, Some("text/plain"), Some("./resumes"), b"redirect");
    let client = rocket_client(relative_mock.port);
    let response = client.get("/resume_editor").dispatch();
    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(
        response.headers().get_one("Location"),
        Some("/resume_editor/resumes")
    );

    let absolute_mock = MockUpstreamHttp::with_response(
        303,
        Some("text/plain"),
        Some("/resume_editor/auth/login"),
        b"redirect",
    );
    let client = rocket_client(absolute_mock.port);
    let response = client.get("/resume_editor").dispatch();
    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(
        response.headers().get_one("Location"),
        Some("/resume_editor/auth/login")
    );

    let cross_mock = MockUpstreamHttp::with_response(
        303,
        Some("text/plain"),
        Some("http://example.com/foo"),
        b"redirect",
    );
    let client = rocket_client(cross_mock.port);
    let response = client.get("/resume_editor").dispatch();
    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(
        response.headers().get_one("Location"),
        Some("http://example.com/foo")
    );
}

#[test]
fn test_upstream_down_returns_502() {
    let unused_port = find_free_port();
    let client = rocket_client(unused_port);

    let response = client.get("/resume_editor/missing").dispatch();

    assert_eq!(response.status(), Status::BadGateway);
}
