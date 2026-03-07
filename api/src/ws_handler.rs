use crate::realtime::Hub;
use rocket::State;
use rocket::futures::{SinkExt, StreamExt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ClientMessage {
    #[serde(rename = "type")]
    kind: String,
    resume_id: Option<i32>,
    token: Option<String>,
}

#[utoipa::path(
    get,
    path = "/ws",
    tag = "WebSocket",
    responses(
        (status = 101, description = "WebSocket Upgrade Successful"),
        (status = 400, description = "Bad Request - Invalid message format"),
        (status = 403, description = "Forbidden - Invalid token or unauthorized")
    )
)]
#[rocket::get("/ws")]
pub fn ws_handler(ws: ws::WebSocket, hub: &State<Hub>) -> ws::Channel<'static> {
    let hub = hub.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let first_message = match stream.next().await {
                Some(Ok(msg)) => msg,
                Some(Err(_)) | None => return Ok(()),
            };

            let subscribe_text = match first_message {
                ws::Message::Text(t) => t,
                ws::Message::Binary(b) => match String::from_utf8(b) {
                    Ok(t) => t,
                    Err(_) => return Ok(()),
                },
                _ => return Ok(()),
            };

            let subscribe: ClientMessage = match serde_json::from_str(&subscribe_text) {
                Ok(v) => v,
                Err(_) => {
                    let _ = stream
                        .send(ws::Message::Text(
                            r#"{"type":"error","message":"invalid_message"}"#.to_string(),
                        ))
                        .await;
                    return Ok(());
                }
            };

            if subscribe.kind != "subscribe" {
                let _ = stream
                    .send(ws::Message::Text(
                        r#"{"type":"error","message":"expected_subscribe"}"#.to_string(),
                    ))
                    .await;
                return Ok(());
            }

            let mut current_resume_id = match subscribe.resume_id {
                Some(v) => v,
                None => {
                    let _ = stream
                        .send(ws::Message::Text(
                            r#"{"type":"error","message":"missing_resume_id"}"#.to_string(),
                        ))
                        .await;
                    return Ok(());
                }
            };

            if !authorize_subscription(current_resume_id, subscribe.token.clone()).await {
                let _ = stream
                    .send(ws::Message::Text(
                        r#"{"type":"error","message":"forbidden"}"#.to_string(),
                    ))
                    .await;
                return Ok(());
            }

            let mut rx = hub.subscribe(current_resume_id);

            loop {
                rocket::tokio::select! {
                    maybe_message = stream.next() => {
                        let Some(message) = maybe_message else {
                            return Ok(());
                        };

                        let message = match message {
                            Ok(m) => m,
                            Err(_) => return Ok(()),
                        };

                        match message {
                            ws::Message::Close(_) => return Ok(()),
                            ws::Message::Text(text) => {
                                if let Ok(cmd) = serde_json::from_str::<ClientMessage>(&text) {
                                    if cmd.kind == "unsubscribe" {
                                        return Ok(());
                                    }

                                    if cmd.kind == "subscribe" {
                                        let Some(next_resume_id) = cmd.resume_id else {
                                            continue;
                                        };

                                        if next_resume_id != current_resume_id {
                                            if authorize_subscription(next_resume_id, cmd.token.clone()).await {
                                                current_resume_id = next_resume_id;
                                                rx = hub.subscribe(current_resume_id);
                                            } else {
                                                let _ = stream
                                                    .send(ws::Message::Text(
                                                        r#"{"type":"error","message":"forbidden"}"#.to_string(),
                                                    ))
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    recv_result = rx.recv() => {
                        let evt = match recv_result {
                            Ok(v) => v,
                            Err(_) => continue,
                        };

                        let payload = match serde_json::to_string(&evt) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };

                        if stream.send(ws::Message::Text(payload)).await.is_err() {
                            return Ok(());
                        }
                    }
                }
            }
        })
    })
}

async fn authorize_subscription(resume_id: i32, token: Option<String>) -> bool {
    rocket::tokio::task::spawn_blocking(move || {
        let user_id = match token {
            Some(t) => {
                let session_id = match t.parse::<uuid::Uuid>() {
                    Ok(v) => v,
                    Err(_) => return false,
                };

                match application::auth::me::resolve_session_user_id(session_id) {
                    Ok(Some(uid)) => Some(uid),
                    Ok(None) => return false,
                    Err(_) => return false,
                }
            }
            None => None,
        };

        match application::resume::read::list_resume(resume_id, user_id) {
            Ok(_) => true,
            Err(_) => false,
        }
    })
    .await
    .unwrap_or(false)
}
