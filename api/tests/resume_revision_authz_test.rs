use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use uuid::Uuid;

mod support;

const LOCK_KEY: i64 = 9_242_020;

fn create_resume(fixture: &support::Fixture, name: &str, visibility: &str) -> Uuid {
    let payload = serde_json::json!({
        "name": name,
        "profile_image_url": null,
        "location": "Here",
        "email": support::unique_email("revision.authz"),
        "github_url": null,
        "mobile_number": null,
        "visibility": visibility
    });

    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Created, "create resume");

    let body: Value = serde_json::from_str(&response.into_string().expect("create body"))
        .expect("valid json");
    support::resume_uuid(&body["body"]["id"])
}

fn get_status(client: &Client, path: String, header: Option<Header<'static>>) -> (Status, Value) {
    let mut request = client.get(path);
    if let Some(header) = header {
        request = request.header(header);
    }
    let response = request.dispatch();
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn put_location(fixture: &support::Fixture, resume_id: Uuid, location: &str) -> Status {
    fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "location": location }).to_string())
        .dispatch()
        .status()
}

#[test]
fn history_routes_require_authentication() {
    let mut fixture = support::Fixture::new(LOCK_KEY);
    let resume_id = create_resume(&fixture, "Private Subject", "private");
    fixture.track_resume_id(resume_id);

    assert_eq!(put_location(&fixture, resume_id, "Edited"), Status::Ok);

    let paths = [
        format!("/api/resume/{}/revisions", resume_id),
        format!("/api/resume/{}/revisions/1", resume_id),
        format!("/api/resume/{}/revisions/1/diff", resume_id),
    ];

    for path in paths {
        let (status, _) = get_status(fixture.client(), path.clone(), None);
        assert_eq!(status, Status::Unauthorized, "anonymous GET {}", path);
    }

    let restore = fixture
        .client()
        .post(format!("/api/resume/{}/revisions/1/restore", resume_id))
        .header(ContentType::JSON)
        .body(serde_json::json!({ "expected_revision": 1 }).to_string())
        .dispatch();
    assert_eq!(restore.status(), Status::Unauthorized);
}

#[test]
fn history_routes_forbid_a_non_owner() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 1);
    let resume_id = create_resume(&fixture, "Owner Only", "private");
    fixture.track_resume_id(resume_id);
    assert_eq!(put_location(&fixture, resume_id, "Edited"), Status::Ok);

    let other = support::register_and_login(fixture.client(), "revision.intruder");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());
    let other_header = support::auth_header(&other.token);

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions", resume_id),
        Some(other_header.clone()),
    );
    assert_eq!(status, Status::Forbidden);

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions/1", resume_id),
        Some(other_header.clone()),
    );
    assert_eq!(status, Status::Forbidden);

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions/1/diff", resume_id),
        Some(other_header.clone()),
    );
    assert_eq!(status, Status::Forbidden);

    let restore = fixture
        .client()
        .post(format!("/api/resume/{}/revisions/1/restore", resume_id))
        .header(other_header)
        .header(ContentType::JSON)
        .body(serde_json::json!({ "expected_revision": 1 }).to_string())
        .dispatch();
    assert_eq!(restore.status(), Status::Forbidden);
}

#[test]
fn history_routes_404_on_an_absent_resume() {
    let fixture = support::Fixture::new(LOCK_KEY + 2);
    let absent = Uuid::now_v7();

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions", absent),
        Some(fixture.auth_header()),
    );
    assert_eq!(status, Status::NotFound);

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions/1", absent),
        Some(fixture.auth_header()),
    );
    assert_eq!(status, Status::NotFound);

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions/1/diff", absent),
        Some(fixture.auth_header()),
    );
    assert_eq!(status, Status::NotFound);

    let restore = fixture
        .client()
        .post(format!("/api/resume/{}/revisions/1/restore", absent))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "expected_revision": 1 }).to_string())
        .dispatch();
    assert_eq!(restore.status(), Status::NotFound);
}

#[test]
fn history_is_owner_only_on_a_readable_resume() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 3);

    for visibility in ["public", "unlisted"] {
        let resume_id = create_resume(&fixture, "Readable Subject", visibility);
        fixture.track_resume_id(resume_id);
        assert_eq!(put_location(&fixture, resume_id, "Edited"), Status::Ok);

        let (status, _) = get_status(
            fixture.client(),
            format!("/api/resume/{}/revisions", resume_id),
            None,
        );
        assert_eq!(
            status,
            Status::Unauthorized,
            "an anonymous reader of a {} resume still cannot read its history",
            visibility
        );

        let other = support::register_and_login(fixture.client(), "revision.reader");
        fixture.track_user_id(other.user_id);
        fixture.track_session_id(other.token.clone());

        let (status, _) = get_status(
            fixture.client(),
            format!("/api/resume/{}/revisions", resume_id),
            Some(support::auth_header(&other.token)),
        );
        assert_eq!(
            status,
            Status::Forbidden,
            "holding the link to a {} resume does not reveal its history",
            visibility
        );
    }
}

#[test]
fn timeline_lists_summaries_newest_first() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 4);
    let resume_id = create_resume(&fixture, "Timeline", "private");
    fixture.track_resume_id(resume_id);

    for location in ["L1", "L2", "L3"] {
        assert_eq!(put_location(&fixture, resume_id, location), Status::Ok);
    }

    let (status, body) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions", resume_id),
        Some(fixture.auth_header()),
    );
    assert_eq!(status, Status::Ok);

    let revisions: Vec<i64> = body["body"]
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry["revision"].as_i64().expect("revision"))
        .collect();
    assert_eq!(revisions, vec![3, 2, 1]);

    let entry = &body["body"][0];
    for field in ["revision", "created_at", "created_by", "action", "is_restore"] {
        assert!(
            entry.get(field).is_some(),
            "a summary carries {}",
            field
        );
    }
    assert!(
        entry.get("snapshot").is_none(),
        "the timeline does not ship snapshot bodies"
    );
}

#[test]
fn out_of_range_revisions_are_404() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 5);
    let resume_id = create_resume(&fixture, "Ranges", "private");
    fixture.track_resume_id(resume_id);
    assert_eq!(put_location(&fixture, resume_id, "Edited"), Status::Ok);

    for path in [
        format!("/api/resume/{}/revisions/0", resume_id),
        format!("/api/resume/{}/revisions/2", resume_id),
        format!("/api/resume/{}/revisions/0/diff", resume_id),
        format!("/api/resume/{}/revisions/2/diff", resume_id),
    ] {
        let (status, _) = get_status(fixture.client(), path.clone(), Some(fixture.auth_header()));
        assert_eq!(status, Status::NotFound, "GET {}", path);
    }
}

#[test]
fn malformed_path_params_are_422() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 6);
    let resume_id = create_resume(&fixture, "Malformed", "private");
    fixture.track_resume_id(resume_id);

    let (status, _) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions/not-a-number", resume_id),
        Some(fixture.auth_header()),
    );
    assert_eq!(status, Status::UnprocessableEntity);

    let (status, _) = get_status(
        fixture.client(),
        "/api/resume/not-a-uuid/revisions".to_string(),
        Some(fixture.auth_header()),
    );
    assert_eq!(status, Status::UnprocessableEntity);
}

#[test]
fn variant_and_base_chains_stay_separate() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 7);
    let base_id = create_resume(&fixture, "Chain Base", "private");
    fixture.track_resume_id(base_id);
    assert_eq!(put_location(&fixture, base_id, "Base Edit"), Status::Ok);

    let variant_response = fixture
        .client()
        .post(format!("/api/resume/{}/variants", base_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "variant_label": "Acme" }).to_string())
        .dispatch();
    assert_eq!(variant_response.status(), Status::Created);
    let variant_body: Value =
        serde_json::from_str(&variant_response.into_string().expect("variant body"))
            .expect("valid json");
    let variant_id = support::resume_uuid(&variant_body["body"]["id"]);
    fixture.track_resume_id(variant_id);

    assert_eq!(
        put_location(&fixture, variant_id, "Variant Edit"),
        Status::Ok
    );

    let (_, base_chain) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions", base_id),
        Some(fixture.auth_header()),
    );
    let base_numbers: Vec<i64> = base_chain["body"]
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry["revision"].as_i64().expect("revision"))
        .collect();
    assert_eq!(
        base_numbers,
        vec![1],
        "a variant edit does not enter the base chain"
    );

    let (_, variant_chain) = get_status(
        fixture.client(),
        format!("/api/resume/{}/revisions", variant_id),
        Some(fixture.auth_header()),
    );
    let variant_numbers: Vec<i64> = variant_chain["body"]
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry["revision"].as_i64().expect("revision"))
        .collect();
    assert_eq!(
        variant_numbers,
        vec![1],
        "a variant clone writes no revision, so its chain starts at its first edit"
    );
}
