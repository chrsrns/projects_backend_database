use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn assert_envelope(status: Status, body_text: &str, expected: Status) {
    assert_eq!(status, expected);
    let json: Value = serde_json::from_str(body_text).expect("catcher emits JSON, not HTML");
    assert!(
        json["body"].is_string(),
        "{expected} body is Response<String>"
    );
}

/// Framework-level rejections under /api keep the Response<String>
/// envelope (V250). Every trigger here is rejected by a Rocket guard or
/// data limit before a handler runs.
#[test]
fn framework_rejections_emit_json_envelope() {
    let mut fixture = support::Fixture::new(9_270_001);

    // 400 — malformed JSON syntax fails the Json<T> data guard.
    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(r#"{"name":"#.to_string())
        .dispatch();
    let status = response.status();
    assert_envelope(status, &response.into_string().unwrap(), Status::BadRequest);

    // 401 — a strict-auth route with no bearer.
    let response = fixture.client().get("/api/auth/me").dispatch();
    let status = response.status();
    assert_envelope(
        status,
        &response.into_string().unwrap(),
        Status::Unauthorized,
    );

    // 413 — markdown body above the 1 MiB limit.
    let big = "x".repeat(1_100_000);
    let response = fixture
        .client()
        .post("/api/resume/validate/markdown")
        .header(ContentType::new("text", "markdown"))
        .body(big)
        .dispatch();
    let status = response.status();
    assert_envelope(
        status,
        &response.into_string().unwrap(),
        Status::PayloadTooLarge,
    );

    // 415 has no live trigger today: every data route declares `format`,
    // so a Content-Type mismatch forwards to 404. The catcher is covered
    // by the unit test in catchers.rs.

    // 422 — valid JSON syntax, wrong shape, fails deserialization.
    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body("{}".to_string())
        .dispatch();
    let status = response.status();
    assert_envelope(
        status,
        &response.into_string().unwrap(),
        Status::UnprocessableEntity,
    );
}

/// A wrong Content-Type on a format-declaring route forwards to 404 —
/// the reachable answer today (415/404/405 stay outside V250's scope).
#[test]
fn wrong_content_type_forwards_to_404() {
    let fixture = support::Fixture::new(9_270_002);
    let resp = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::Plain)
        .body("name=X")
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}
