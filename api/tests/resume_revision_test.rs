use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use uuid::Uuid;

mod support;

const LOCK_KEY: i64 = 9_242_001;

fn create_resume(fixture: &support::Fixture, name: &str, location: &str) -> Uuid {
    let payload = serde_json::json!({
        "name": name,
        "profile_image_url": null,
        "location": location,
        "email": support::unique_email("revision.resume"),
        "github_url": null,
        "mobile_number": null,
        "visibility": "private"
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

fn get_json(client: &Client, path: String, header: Header<'static>) -> (Status, Value) {
    let response = client.get(path).header(header).dispatch();
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    let json = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

fn list_revisions(fixture: &support::Fixture, resume_id: Uuid) -> (Status, Value) {
    get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions", resume_id),
        fixture.auth_header(),
    )
}

fn fetch_revision(fixture: &support::Fixture, resume_id: Uuid, revision: i64) -> (Status, Value) {
    get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions/{}", resume_id, revision),
        fixture.auth_header(),
    )
}

fn revision_numbers(body: &Value) -> Vec<i64> {
    body["body"]
        .as_array()
        .expect("revision list body should be an array")
        .iter()
        .map(|entry| entry["revision"].as_i64().expect("revision number"))
        .collect()
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

fn put_education(fixture: &support::Fixture, education_id: i64, payload: Value) -> Status {
    fixture
        .client()
        .put(format!("/api/education/{}", education_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch()
        .status()
}

fn delete_education(fixture: &support::Fixture, education_id: i64) -> Status {
    fixture
        .client()
        .delete(format!("/api/education/{}", education_id))
        .header(fixture.auth_header())
        .dispatch()
        .status()
}

fn create_education(fixture: &support::Fixture, resume_id: Uuid) -> i64 {
    let payload = serde_json::json!({
        "education_stage": "Bachelor",
        "institution_name": "Original University",
        "degree": "BSc",
        "start_date": "2020-01-01",
        "display_order": 1
    });

    let response = fixture
        .client()
        .post(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Created, "create education");

    let body: Value = serde_json::from_str(&response.into_string().expect("education body"))
        .expect("valid json");
    body["body"]["id"].as_i64().expect("education id")
}

#[test]
fn scalar_update_writes_one_revision_with_pre_edit_snapshot() {
    let mut fixture = support::Fixture::new(LOCK_KEY);
    let resume_id = create_resume(&fixture, "Revision Subject", "Before");
    fixture.track_resume_id(resume_id);

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert!(
        revision_numbers(&body).is_empty(),
        "creating a resume has no before-image, so it writes no revision"
    );

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "After" })),
        Status::Ok
    );

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(revision_numbers(&body), vec![1]);

    let summary = &body["body"][0];
    assert_eq!(summary["created_by"].as_i64(), Some(fixture.user_id() as i64));
    assert_eq!(summary["is_restore"].as_bool(), Some(false));
    assert_eq!(summary["action"].as_str(), Some("personalinfo"));

    let (status, revision) = fetch_revision(&fixture, resume_id, 1);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        revision["body"]["snapshot"]["location"].as_str(),
        Some("Before"),
        "revision 1 holds the state before the edit"
    );
    assert_eq!(
        revision["body"]["snapshot"]["name"].as_str(),
        Some("Revision Subject")
    );
}

#[test]
fn child_create_update_delete_each_write_one_revision() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 1);
    let resume_id = create_resume(&fixture, "Child Edits", "Here");
    fixture.track_resume_id(resume_id);

    let education_id = create_education(&fixture, resume_id);
    assert_eq!(
        put_education(
            &fixture,
            education_id,
            serde_json::json!({ "institution_name": "Renamed University" })
        ),
        Status::Ok
    );
    assert_eq!(delete_education(&fixture, education_id), Status::NoContent);

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(revision_numbers(&body), vec![3, 2, 1]);

    let (_, first) = fetch_revision(&fixture, resume_id, 1);
    assert_eq!(
        first["body"]["snapshot"]["education"]
            .as_array()
            .expect("education array")
            .len(),
        0,
        "revision 1 predates the education row"
    );

    let (_, second) = fetch_revision(&fixture, resume_id, 2);
    assert_eq!(
        second["body"]["snapshot"]["education"][0]["institution_name"].as_str(),
        Some("Original University")
    );

    let (_, third) = fetch_revision(&fixture, resume_id, 3);
    assert_eq!(
        third["body"]["snapshot"]["education"][0]["institution_name"].as_str(),
        Some("Renamed University")
    );
}

#[test]
fn noop_update_writes_no_revision() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 2);
    let resume_id = create_resume(&fixture, "No Op", "Same");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({})),
        Status::Ok,
        "an all-absent patch is a no-op, not an error"
    );

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert!(
        revision_numbers(&body).is_empty(),
        "a mutation that executes no write captures nothing"
    );
}

#[test]
fn resume_create_and_delete_write_no_revision() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 3);
    let resume_id = create_resume(&fixture, "Short Lived", "Gone");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "Kept" })),
        Status::Ok
    );
    let (_, body) = list_revisions(&fixture, resume_id);
    assert_eq!(revision_numbers(&body), vec![1]);

    let delete_status = fixture
        .client()
        .delete(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .dispatch()
        .status();
    assert_eq!(delete_status, Status::NoContent);

    let (status, _) = list_revisions(&fixture, resume_id);
    assert_eq!(
        status,
        Status::NotFound,
        "deleting a resume removes its row and its trail"
    );
}

#[test]
fn revision_numbers_are_dense_from_one() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 4);
    let resume_id = create_resume(&fixture, "Dense", "L0");
    fixture.track_resume_id(resume_id);

    for location in ["L1", "L2", "L3", "L4"] {
        assert_eq!(
            put_resume(
                &fixture,
                resume_id,
                serde_json::json!({ "location": location })
            ),
            Status::Ok
        );
    }

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(revision_numbers(&body), vec![4, 3, 2, 1]);

    for revision in 1..=4 {
        let (status, _) = fetch_revision(&fixture, resume_id, revision);
        assert_eq!(status, Status::Ok, "revision {} exists", revision);
    }

    let (status, _) = fetch_revision(&fixture, resume_id, 5);
    assert_eq!(status, Status::NotFound);
}
