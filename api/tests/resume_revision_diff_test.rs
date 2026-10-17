use application::error::ApplicationError;
use application::realtime::{ResumeChangePublisher, ResumeChangedAction};
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use uuid::Uuid;

mod support;

const LOCK_KEY: i64 = 9_242_040;

struct NullPublisher;

impl ResumeChangePublisher for NullPublisher {
    fn resume_changed(&self, _resume_id: Uuid, _action: ResumeChangedAction) {}
}

fn create_resume(fixture: &support::Fixture, name: &str) -> Uuid {
    let created = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": name,
                "profile_image_url": null,
                "location": "L0",
                "email": support::unique_email("revision.diff"),
                "github_url": null,
                "mobile_number": null,
                "visibility": "private"
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(created.status(), Status::Created, "create resume");
    let body: Value = serde_json::from_str(&created.into_string().expect("body")).expect("json");
    support::resume_uuid(&body["body"]["id"])
}

fn get_json(client: &Client, path: String, header: Header<'static>) -> (Status, Value) {
    let response = client.get(path).header(header).dispatch();
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn put_resume(fixture: &support::Fixture, resume_id: Uuid, payload: Value) -> Status {
    fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch()
        .status()
}

fn head_revision(fixture: &support::Fixture, resume_id: Uuid) -> i64 {
    let (_, body) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions", resume_id),
        fixture.auth_header(),
    );
    body["body"][0]["revision"]
        .as_i64()
        .expect("the resume has a revision")
}

fn restore(
    fixture: &support::Fixture,
    resume_id: Uuid,
    revision: i64,
    expected: i64,
) -> (Status, Value) {
    let response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/revisions/{}/restore",
            resume_id, revision
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "expected_revision": expected }).to_string())
        .dispatch();

    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn add_education_with_key_point(fixture: &support::Fixture, resume_id: Uuid) {
    let education = fixture
        .client()
        .post(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "education_stage": "Bachelor",
                "institution_name": "University",
                "start_date": "2020-01-01",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(education.status(), Status::Created);
    let body: Value = serde_json::from_str(&education.into_string().expect("body")).expect("json");
    let education_id = body["body"]["id"].as_i64().expect("education id");

    let key_point = fixture
        .client()
        .post(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({ "key_point": "Graduated with honors", "display_order": 1 })
                .to_string(),
        )
        .dispatch();
    assert_eq!(key_point.status(), Status::Created);
}

#[test]
fn diff_across_a_restore_boundary_has_no_child_churn() {
    let mut fixture = support::Fixture::new(LOCK_KEY);
    let resume_id = create_resume(&fixture, "Churn Subject");
    fixture.track_resume_id(resume_id);

    add_education_with_key_point(&fixture, resume_id);
    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );

    let head = head_revision(&fixture, resume_id);
    let (status, _) = restore(&fixture, resume_id, head, head);
    assert_eq!(status, Status::Ok);

    // The appended revision holds the state the restore replaced, whose
    // children carried the ids from before the restore. The live state has
    // fresh ids and identical content, so the diff must be empty: comparing by
    // stored id would report every child as removed and re-added.
    let appended = head + 1;
    let (status, diff) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions/{}/diff", resume_id, appended),
        fixture.auth_header(),
    );
    assert_eq!(status, Status::Ok);

    let changes = diff["body"]["changes"].as_array().expect("changes array");

    let child_collections = [
        "/education",
        "/education_key_points",
        "/skills",
        "/work_experiences",
        "/work_experience_key_points",
        "/portfolio_projects",
        "/portfolio_key_points",
        "/portfolio_technologies",
        "/languages",
        "/frameworks",
    ];

    for change in changes {
        let path = change["path"].as_str().expect("path");
        let kind = change["kind"].as_str().expect("kind");

        assert_ne!(
            kind, "added",
            "the restore adds nothing, but reported {}",
            path
        );
        assert_ne!(
            kind, "removed",
            "the restore removes nothing, but reported {}",
            path
        );
        assert!(
            !child_collections
                .iter()
                .any(|collection| path.starts_with(collection)),
            "renumbered children with identical content are not a change, but reported {}",
            path
        );
    }

    assert!(
        changes
            .iter()
            .any(|change| change["path"].as_str() == Some("/location")),
        "the restore's own content change is still reported: {:?}",
        changes
    );
}

#[test]
fn restore_appended_revision_records_action_and_flag() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 1);
    let resume_id = create_resume(&fixture, "Flag Subject");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );

    let head = head_revision(&fixture, resume_id);
    let (status, restored) = restore(&fixture, resume_id, head, head);
    assert_eq!(status, Status::Ok);
    assert_eq!(restored["body"]["location"].as_str(), Some("L0"));

    let (_, timeline) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions", resume_id),
        fixture.auth_header(),
    );
    let entries = timeline["body"].as_array().expect("array");

    assert_eq!(entries[0]["revision"].as_i64(), Some(head + 1));
    assert_eq!(entries[0]["is_restore"].as_bool(), Some(true));
    assert_eq!(entries[0]["action"].as_str(), Some("personalinfo"));
    assert_eq!(
        entries[1]["is_restore"].as_bool(),
        Some(false),
        "only the restore-appended revision is flagged"
    );
}

#[test]
fn base_restore_with_a_taken_email_is_409() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 2);
    let resume_id = create_resume(&fixture, "Email Subject");
    fixture.track_resume_id(resume_id);

    let (_, created) = get_json(
        fixture.client(),
        format!("/api/resume/{}", resume_id),
        fixture.auth_header(),
    );
    let original_email = created["body"]["email"]
        .as_str()
        .expect("email")
        .to_string();

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );
    let replacement_email = support::unique_email("revision.replacement");
    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "email": replacement_email })
        ),
        Status::Ok
    );

    // A second base resume now claims the address the first one used to hold.
    let claimant = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Claimant",
                "profile_image_url": null,
                "location": "Elsewhere",
                "email": original_email,
                "github_url": null,
                "mobile_number": null,
                "visibility": "private"
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(claimant.status(), Status::Created);
    let claimant_body: Value =
        serde_json::from_str(&claimant.into_string().expect("body")).expect("json");
    fixture.track_resume_id(support::resume_uuid(&claimant_body["body"]["id"]));

    let head = head_revision(&fixture, resume_id);
    let (status, _) = restore(&fixture, resume_id, head, head);
    assert_eq!(
        status,
        Status::Conflict,
        "restoring an email another base resume now holds is refused"
    );

    let (_, current) = get_json(
        fixture.client(),
        format!("/api/resume/{}", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(
        current["body"]["email"].as_str(),
        Some(replacement_email.as_str()),
        "a refused restore writes nothing"
    );
    assert_eq!(
        head_revision(&fixture, resume_id),
        head,
        "a refused restore appends nothing"
    );
}

#[test]
fn unified_diff_only_when_requested() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 3);
    let resume_id = create_resume(&fixture, "Unified Subject");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );

    let (status, plain) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions/1/diff", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(status, Status::Ok);
    assert!(
        plain["body"].get("unified").is_none(),
        "the unified rendering is omitted unless asked for"
    );

    let (status, unified) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions/1/diff?unified=true", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(status, Status::Ok);
    let rendered = unified["body"]["unified"]
        .as_str()
        .expect("the unified rendering is present");
    assert!(rendered.contains("--- before"));
    assert!(rendered.contains("L0"));
    assert!(rendered.contains("L1"));

    let (status, _) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions/1/diff?unified=yes", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(
        status,
        Status::BadRequest,
        "an unrecognised value is refused rather than ignored"
    );
}

#[test]
fn restore_racing_an_edit_keeps_the_chain_dense() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 4);
    let user_id = fixture.user_id();
    let resume_id = create_resume(&fixture, "Race Subject");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );

    let head = head_revision(&fixture, resume_id) as i32;

    let edit = std::thread::spawn(move || {
        let dto: domain::models::UpdateResume =
            serde_json::from_value(serde_json::json!({ "location": "L2" })).expect("dto");

        application::resume::update::update_resume(user_id, resume_id, dto, &NullPublisher).is_ok()
    });

    let restore_outcome = application::resume::revision::restore::restore_revision(
        user_id,
        resume_id,
        head,
        head,
        &NullPublisher,
    );

    let edit_succeeded = edit.join().expect("thread joins");

    let (_, timeline) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions", resume_id),
        fixture.auth_header(),
    );
    let numbers: Vec<i64> = timeline["body"]
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry["revision"].as_i64().expect("revision"))
        .collect();

    let expected_len = head as i64 + edit_succeeded as i64 + restore_outcome.is_ok() as i64;
    assert_eq!(
        numbers.len() as i64,
        expected_len,
        "every successful write is recorded exactly once: {:?}",
        numbers
    );

    let dense: Vec<i64> = (1..=expected_len).rev().collect();
    assert_eq!(numbers, dense, "numbering stays dense");

    // A refused restore is refused for the guard, never for a collision.
    if let Err(err) = restore_outcome {
        assert!(
            matches!(err, ApplicationError::Conflict(_)),
            "a racing restore is refused by the guard, got {:?}",
            err
        );
    }
}
