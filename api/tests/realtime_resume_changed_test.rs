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
        "is_public": true
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
    let resume_id = create_json["body"]["id"].as_i64().expect("id") as i32;

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
        "is_public": false
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
    let resume_id = create_json["body"]["id"].as_i64().expect("id") as i32;
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
        "is_public": false
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
    let resume_id = create_json["body"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(resume_id);

    let port = 18_000 + (resume_id as u16 % 1_000);
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
    assert_eq!(event_json["resume_id"], resume_id);
    assert_eq!(event_json["action"]["updated"], "personalinfo");

    runtime.block_on(async move {
        shutdown.notify();
        let _ = server.await;
    });
}
