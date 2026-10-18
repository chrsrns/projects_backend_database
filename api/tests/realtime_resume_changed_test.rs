use futures_util::{SinkExt, StreamExt};
use rocket::http::{ContentType, Status};
use serde_json::Value;
use std::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use api::realtime::ResumeChangedAction;

mod support;

#[test]
fn test_resume_update_publishes_resume_changed_event() {
    let mut fixture = support::Fixture::new(9_225_339);

    let unique_email = format!(
        "realtime.resume.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Realtime User",
        "profile_image_url": null,
        "location": "Initial",
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "visibility": "public"
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid json");
    let resume_id = support::resume_uuid(&create_json["body"]["id"]);

    fixture.track_resume_id(resume_id);

    let mut rx = fixture.hub.subscribe(resume_id);

    let update_payload = serde_json::json!({
        "location": "Updated"
    });

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_payload.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let evt = rx.try_recv().expect("resume.changed event");
    assert_eq!(evt.resume_id, resume_id);
    assert_eq!(
        evt.action,
        ResumeChangedAction::Updated(api::realtime::SectionType::PersonalInfo)
    );
}

#[test]
fn test_skill_create_publishes_resume_changed_event() {
    let mut fixture = support::Fixture::new(9_225_340);

    let unique_email = format!(
        "realtime.skills.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Realtime Skills User",
        "profile_image_url": null,
        "location": "Initial",
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "visibility": "private"
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid json");
    let resume_id = support::resume_uuid(&create_json["body"]["id"]);
    fixture.track_resume_id(resume_id);

    let mut rx = fixture.hub.subscribe(resume_id);

    let new_skill_json = serde_json::json!({
        "skill_name": "Rust",
        "confidence_percentage": 80,
        "display_order": 0
    });

    let create_skill_response = fixture
        .client()
        .post(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_skill_json.to_string())
        .dispatch();

    assert_eq!(create_skill_response.status(), Status::Created);

    let evt = rx.try_recv().expect("resume.changed event");
    assert_eq!(evt.resume_id, resume_id);
    assert_eq!(
        evt.action,
        ResumeChangedAction::Updated(api::realtime::SectionType::Skills)
    );
}

#[test]
fn test_ws_endpoint_subscribe_receives_resume_changed_event() {
    let mut fixture = support::Fixture::new(9_225_341);

    let unique_email = format!(
        "realtime.ws.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Realtime Ws User",
        "profile_image_url": null,
        "location": "Initial",
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "visibility": "private"
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid json");
    let resume_id = support::resume_uuid(&create_json["body"]["id"]);
    fixture.track_resume_id(resume_id);

    let port = 18_000 + (resume_id.as_u128() % 1_000) as u16;
    let ws_token = fixture.auth_token().to_string();
    let hub = fixture.hub.clone();
    let (event_tx, event_rx) = mpsc::channel();

    let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");
    let (shutdown, server) = runtime.block_on(async move {
        let rocket =
            api::build_rocket_with_hub(hub, shared::node_config::NodeConfig { port: 53421 })
                .configure(
                    rocket::Config::figment()
                        .merge(("address", "127.0.0.1"))
                        .merge(("port", port)),
                );

        let ignited = rocket.ignite().await.expect("ignite rocket");
        let shutdown = ignited.shutdown();
        let server = rocket::tokio::spawn(async move {
            let _ = ignited.launch().await;
        });

        rocket::tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let ws_url = format!("ws://127.0.0.1:{}/api/ws", port);
        let (mut socket, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("connect websocket");

        let subscribe_message = serde_json::json!({
            "type": "subscribe",
            "resume_id": resume_id,
            "token": ws_token
        });

        socket
            .send(Message::Text(subscribe_message.to_string().into()))
            .await
            .expect("send subscribe message");

        rocket::tokio::spawn(async move {
            let incoming =
                rocket::tokio::time::timeout(std::time::Duration::from_secs(2), socket.next())
                    .await
                    .expect("websocket message timeout")
                    .expect("websocket stream item")
                    .expect("websocket text frame");

            let payload = match incoming {
                Message::Text(text) => text.to_string(),
                other => panic!("unexpected websocket message: {other:?}"),
            };

            event_tx
                .send(payload)
                .expect("send websocket event to test thread");
        });

        (shutdown, server)
    });

    std::thread::sleep(std::time::Duration::from_millis(50));

    let update_payload = serde_json::json!({
        "location": "Updated over websocket"
    });

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_payload.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let payload = event_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .expect("receive websocket event");

    let event_json: Value = serde_json::from_str(&payload).expect("valid event json");
    assert_eq!(event_json["type"], "resume.changed");
    assert_eq!(event_json["resume_id"], resume_id.to_string());
    assert_eq!(event_json["action"]["updated"], "personalinfo");

    runtime.block_on(async move {
        shutdown.notify();
        let _ = server.await;
    });
}

#[test]
fn test_ws_subscribe_unlisted_resume_anonymous() {
    let mut fixture = support::Fixture::new(9_226_020);

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Realtime Unlisted",
                "email": support::unique_email("realtime.unlisted"),
                "visibility": "unlisted"
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(create_response.status(), Status::Created);
    let create_json: Value = serde_json::from_str(&create_response.into_string().unwrap()).unwrap();
    let resume_id = support::resume_uuid(&create_json["body"]["id"]);
    fixture.track_resume_id(resume_id);

    let port = 18_000 + (resume_id.as_u128() % 1_000) as u16;
    let hub = fixture.hub.clone();
    let (event_tx, event_rx) = mpsc::channel();

    let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");
    let (shutdown, server) = runtime.block_on(async move {
        let rocket =
            api::build_rocket_with_hub(hub, shared::node_config::NodeConfig { port: 53421 })
                .configure(
                    rocket::Config::figment()
                        .merge(("address", "127.0.0.1"))
                        .merge(("port", port)),
                );

        let ignited = rocket.ignite().await.expect("ignite rocket");
        let shutdown = ignited.shutdown();
        let server = rocket::tokio::spawn(async move {
            let _ = ignited.launch().await;
        });

        rocket::tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let ws_url = format!("ws://127.0.0.1:{}/api/ws", port);
        let (mut socket, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("connect websocket");

        // No token: the link itself is the capability for an unlisted resume.
        let subscribe_message = serde_json::json!({
            "type": "subscribe",
            "resume_id": resume_id,
            "token": null
        });

        socket
            .send(Message::Text(subscribe_message.to_string().into()))
            .await
            .expect("send subscribe message");

        rocket::tokio::spawn(async move {
            let incoming =
                rocket::tokio::time::timeout(std::time::Duration::from_secs(2), socket.next())
                    .await
                    .expect("websocket message timeout")
                    .expect("websocket stream item")
                    .expect("websocket text frame");

            let payload = match incoming {
                Message::Text(text) => text.to_string(),
                other => panic!("unexpected websocket message: {other:?}"),
            };

            event_tx
                .send(payload)
                .expect("send websocket event to test thread");
        });

        (shutdown, server)
    });

    std::thread::sleep(std::time::Duration::from_millis(50));

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({"location": "Updated"}).to_string())
        .dispatch();
    assert_eq!(update_response.status(), Status::Ok);

    let payload = event_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .expect("receive websocket event");

    let event_json: Value = serde_json::from_str(&payload).expect("valid event json");
    assert_eq!(event_json["type"], "resume.changed");
    assert_eq!(event_json["resume_id"], resume_id.to_string());

    runtime.block_on(async move {
        shutdown.notify();
        let _ = server.await;
    });
}

/// A live `/api/ws` connection driven from the test thread.
///
/// The socket itself stays inside the runtime; the test thread talks to it
/// through a command channel in and a frame channel out, so a test can send
/// more than one subscribe and read more than one frame from one connection.
struct WsHarness {
    runtime: rocket::tokio::runtime::Runtime,
    shutdown: rocket::Shutdown,
    server: rocket::tokio::task::JoinHandle<()>,
    commands: rocket::tokio::sync::mpsc::UnboundedSender<String>,
    frames: mpsc::Receiver<String>,
}

impl WsHarness {
    /// Boots a Rocket instance on `port` and opens a socket against it.
    fn open(hub: api::realtime::Hub, port: u16) -> Self {
        let (frame_tx, frame_rx) = mpsc::channel::<String>();
        let (command_tx, mut command_rx) = rocket::tokio::sync::mpsc::unbounded_channel::<String>();

        let runtime = rocket::tokio::runtime::Runtime::new().expect("tokio runtime");

        let (shutdown, server) = runtime.block_on(async move {
            let rocket =
                api::build_rocket_with_hub(hub, shared::node_config::NodeConfig { port: 53421 })
                    .configure(
                        rocket::Config::figment()
                            .merge(("address", "127.0.0.1"))
                            .merge(("port", port)),
                    );

            let ignited = rocket.ignite().await.expect("ignite rocket");
            let shutdown = ignited.shutdown();
            let server = rocket::tokio::spawn(async move {
                let _ = ignited.launch().await;
            });

            rocket::tokio::time::sleep(std::time::Duration::from_millis(150)).await;

            let ws_url = format!("ws://127.0.0.1:{}/api/ws", port);
            let (socket, _) = tokio_tungstenite::connect_async(&ws_url)
                .await
                .expect("connect websocket");

            let (mut sink, mut stream) = socket.split();

            rocket::tokio::spawn(async move {
                while let Some(text) = command_rx.recv().await {
                    if sink.send(Message::Text(text.into())).await.is_err() {
                        return;
                    }
                }
            });

            rocket::tokio::spawn(async move {
                while let Some(Ok(message)) = stream.next().await {
                    if let Message::Text(text) = message
                        && frame_tx.send(text.to_string()).is_err()
                    {
                        return;
                    }
                }
            });

            (shutdown, server)
        });

        WsHarness {
            runtime,
            shutdown,
            server,
            commands: command_tx,
            frames: frame_rx,
        }
    }

    fn subscribe(&self, message: Value) {
        self.commands
            .send(message.to_string())
            .expect("send subscribe message");
    }

    fn next_frame(&self) -> String {
        self.frames
            .recv_timeout(std::time::Duration::from_secs(3))
            .expect("expected a websocket frame")
    }

    /// Asserts the server sends nothing back within `within`.
    fn expect_no_frame(&self, within: std::time::Duration) {
        match self.frames.recv_timeout(within) {
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            other => panic!("expected no frame, got {other:?}"),
        }
    }

    fn close(self) {
        let WsHarness {
            runtime,
            shutdown,
            server,
            ..
        } = self;

        runtime.block_on(async move {
            shutdown.notify();
            let _ = server.await;
        });
    }
}

fn create_resume_with_visibility(
    fixture: &mut support::Fixture,
    token: &str,
    visibility: &str,
) -> uuid::Uuid {
    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(token))
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": format!("Realtime {visibility}"),
                "email": support::unique_email(&format!("realtime.{visibility}")),
                "visibility": visibility
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Created, "resume create failed");

    let body = response.into_string().expect("create body");
    let json: Value = serde_json::from_str(&body).expect("valid json");
    let resume_id = support::resume_uuid(&json["body"]["id"]);
    fixture.track_resume_id(resume_id);

    resume_id
}

/// Publishes a `resume.changed` event for `resume_id` by updating it.
fn publish_change(fixture: &support::Fixture, token: &str, resume_id: uuid::Uuid) {
    let response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(support::auth_header(token))
        .header(ContentType::JSON)
        .body(serde_json::json!({"location": "Updated over websocket"}).to_string())
        .dispatch();

    assert_eq!(
        response.status(),
        Status::Ok,
        "publishing an update should succeed"
    );
}

/// An expired session row must read as absent, exactly like no token at all,
/// so a public resume's subscription still receives change events.
#[test]
fn ws_subscribe_with_an_expired_token_receives_events_on_a_public_resume() {
    let mut fixture = support::Fixture::new(9_242_007);

    let owner = support::register_and_login(fixture.client(), "ws.expired.public");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    // A second session keeps the owner able to publish once the first one is
    // expired.
    let live_token = support::login_session(fixture.client(), &owner.email);
    fixture.track_session_id(live_token.clone());

    let resume_id = create_resume_with_visibility(&mut fixture, &owner.token, "public");
    fixture.expire_session(&owner.token);

    let harness = WsHarness::open(fixture.hub.clone(), 17_001);
    harness.subscribe(serde_json::json!({
        "type": "subscribe",
        "resume_id": resume_id,
        "token": owner.token
    }));

    std::thread::sleep(std::time::Duration::from_millis(50));
    publish_change(&fixture, &live_token, resume_id);

    let frame = harness.next_frame();
    let json: Value = serde_json::from_str(&frame).expect("valid frame json");
    assert_eq!(
        json["type"], "resume.changed",
        "an expired token must not deny a public subscription: {frame}"
    );
    assert_eq!(json["resume_id"], resume_id.to_string());

    harness.close();
}

/// A token naming no session row, and a token that is not a UUID at all,
/// both read as absent on an unlisted resume.
#[test]
fn ws_subscribe_with_unknown_or_malformed_tokens_receives_events_on_an_unlisted_resume() {
    let mut fixture = support::Fixture::new(9_242_008);

    let owner = support::register_and_login(fixture.client(), "ws.unresolvable.unlisted");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let resume_id = create_resume_with_visibility(&mut fixture, &owner.token, "unlisted");

    let cases = [
        ("unknown uuid", uuid::Uuid::new_v4().to_string(), 17_002_u16),
        ("malformed", "not-a-uuid".to_string(), 17_003_u16),
    ];

    for (label, token, port) in cases {
        let harness = WsHarness::open(fixture.hub.clone(), port);
        harness.subscribe(serde_json::json!({
            "type": "subscribe",
            "resume_id": resume_id,
            "token": token
        }));

        std::thread::sleep(std::time::Duration::from_millis(50));
        publish_change(&fixture, &owner.token, resume_id);

        let frame = harness.next_frame();
        let json: Value = serde_json::from_str(&frame).expect("valid frame json");
        assert_eq!(
            json["type"], "resume.changed",
            "an unresolvable ({label}) token must not deny an unlisted subscription: {frame}"
        );
        assert_eq!(json["resume_id"], resume_id.to_string());

        harness.close();
    }
}

/// An unresolvable token on a private resume is still denied, and denied by
/// the visibility predicate rather than by the token.
#[test]
fn ws_subscribe_with_an_unresolvable_token_is_forbidden_on_a_private_resume() {
    let mut fixture = support::Fixture::new(9_242_009);

    let owner = support::register_and_login(fixture.client(), "ws.unresolvable.private");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let resume_id = create_resume_with_visibility(&mut fixture, &owner.token, "private");

    let expired = support::login_session(fixture.client(), &owner.email);
    fixture.track_session_id(expired.clone());
    fixture.expire_session(&expired);

    let cases = [
        ("expired session row", expired.clone(), 17_004_u16),
        ("unknown uuid", uuid::Uuid::new_v4().to_string(), 17_005_u16),
        ("malformed", "not-a-uuid".to_string(), 17_006_u16),
    ];

    for (label, token, port) in cases {
        let harness = WsHarness::open(fixture.hub.clone(), port);
        harness.subscribe(serde_json::json!({
            "type": "subscribe",
            "resume_id": resume_id,
            "token": token
        }));

        let frame = harness.next_frame();
        let json: Value = serde_json::from_str(&frame).expect("valid frame json");
        assert_eq!(
            json["type"], "error",
            "a private resume with an unresolvable ({label}) token must be denied: {frame}"
        );
        assert_eq!(json["message"], "forbidden");

        harness.close();
    }
}

/// A denied re-subscribe must not tear down the subscription already live on
/// the same connection.
#[test]
fn ws_denied_resubscribe_leaves_the_previous_subscription_live() {
    let mut fixture = support::Fixture::new(9_242_010);

    let owner = support::register_and_login(fixture.client(), "ws.resubscribe");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let public_id = create_resume_with_visibility(&mut fixture, &owner.token, "public");
    let private_id = create_resume_with_visibility(&mut fixture, &owner.token, "private");

    let harness = WsHarness::open(fixture.hub.clone(), 17_007);
    harness.subscribe(serde_json::json!({
        "type": "subscribe",
        "resume_id": public_id,
        "token": owner.token
    }));

    std::thread::sleep(std::time::Duration::from_millis(50));

    harness.subscribe(serde_json::json!({
        "type": "subscribe",
        "resume_id": private_id,
        "token": "not-a-uuid"
    }));

    let denied: Value = serde_json::from_str(&harness.next_frame()).expect("valid frame json");
    assert_eq!(denied["type"], "error");
    assert_eq!(denied["message"], "forbidden");

    // The earlier subscription is still live, so a change on the public
    // resume still reaches this socket.
    publish_change(&fixture, &owner.token, public_id);

    let frame = harness.next_frame();
    let json: Value = serde_json::from_str(&frame).expect("valid frame json");
    assert_eq!(
        json["type"], "resume.changed",
        "a denied re-subscribe must not disconnect the live subscription: {frame}"
    );
    assert_eq!(json["resume_id"], public_id.to_string());

    harness.close();
}

/// Re-subscribing to the resume already subscribed to changes nothing.
///
/// The observable consequence is that the guard is skipped entirely: a
/// second subscribe naming the same resume does not re-authorize, so even a
/// token the guard could not resolve produces no error frame. The live
/// subscription stays and still delivers exactly one event per change.
#[test]
fn ws_resubscribing_to_the_same_resume_is_a_no_op() {
    let mut fixture = support::Fixture::new(9_242_075);

    let owner = support::register_and_login(fixture.client(), "ws.resubscribe.same");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let resume_id = create_resume_with_visibility(&mut fixture, &owner.token, "private");

    let harness = WsHarness::open(fixture.hub.clone(), 17_008);
    harness.subscribe(serde_json::json!({
        "type": "subscribe",
        "resume_id": resume_id,
        "token": owner.token
    }));

    std::thread::sleep(std::time::Duration::from_millis(50));

    harness.subscribe(serde_json::json!({
        "type": "subscribe",
        "resume_id": resume_id,
        "token": "not-a-uuid"
    }));

    harness.expect_no_frame(std::time::Duration::from_millis(500));

    publish_change(&fixture, &owner.token, resume_id);

    let frame = harness.next_frame();
    let json: Value = serde_json::from_str(&frame).expect("valid frame json");
    assert_eq!(
        json["type"], "resume.changed",
        "the original subscription is still the live one: {frame}"
    );
    assert_eq!(json["resume_id"], resume_id.to_string());

    harness.close();
}
