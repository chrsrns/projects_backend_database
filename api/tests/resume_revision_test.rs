use diesel::prelude::*;
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

    let body: Value =
        serde_json::from_str(&response.into_string().expect("create body")).expect("valid json");
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

    let body: Value =
        serde_json::from_str(&response.into_string().expect("education body")).expect("valid json");
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
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "After" })
        ),
        Status::Ok
    );

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(revision_numbers(&body), vec![1]);

    let summary = &body["body"][0];
    assert_eq!(
        summary["created_by"].as_i64(),
        Some(fixture.user_id() as i64)
    );
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
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "Kept" })
        ),
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
fn delete_miss_writes_no_revision() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 18);
    let resume_id = create_resume(&fixture, "Delete Miss", "Here");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "There" })
        ),
        Status::Ok
    );
    let (_, body) = list_revisions(&fixture, resume_id);
    assert_eq!(revision_numbers(&body), vec![1]);

    assert_eq!(
        delete_education(&fixture, 9_999_999),
        Status::NotFound,
        "deleting a row that does not exist is a miss"
    );

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        revision_numbers(&body),
        vec![1],
        "a delete that matched no row leaves no revision"
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

fn post_json(fixture: &support::Fixture, path: String, payload: Value) -> (Status, Value) {
    let response = fixture
        .client()
        .post(path)
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch();

    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn post_child(fixture: &support::Fixture, path: String, payload: Value) -> i64 {
    let label = path.clone();
    let (status, body) = post_json(fixture, path, payload);
    assert_eq!(status, Status::Created, "create child at {}", label);
    body["body"]["id"].as_i64().expect("child id")
}

fn create_child_set(fixture: &support::Fixture, resume_id: Uuid) -> ChildIds {
    let education_id = create_education(fixture, resume_id);

    let education_key_point_id = post_child(
        fixture,
        format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ),
        serde_json::json!({ "key_point": "Graduated with honors", "display_order": 1 }),
    );

    let skill_id = post_child(
        fixture,
        format!("/api/resume/{}/skills", resume_id),
        serde_json::json!({ "skill_name": "Rust", "confidence_percentage": 90, "display_order": 1 }),
    );

    let work_id = post_child(
        fixture,
        format!("/api/resume/{}/work_experiences", resume_id),
        serde_json::json!({
            "job_title": "Engineer",
            "company_name": "Tech Corp",
            "start_date": "2021-01",
            "display_order": 1
        }),
    );

    let work_key_point_id = post_child(
        fixture,
        format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            resume_id, work_id
        ),
        serde_json::json!({ "key_point": "Shipped the API", "display_order": 1 }),
    );

    let project_id = post_child(
        fixture,
        format!("/api/resume/{}/portfolio_projects", resume_id),
        serde_json::json!({ "project_name": "Resume API", "display_order": 1 }),
    );

    let project_key_point_id = post_child(
        fixture,
        format!(
            "/api/resume/{}/portfolio_projects/{}/key_points",
            resume_id, project_id
        ),
        serde_json::json!({ "key_point": "Serves markdown", "display_order": 1 }),
    );

    let technology_id = post_child(
        fixture,
        format!(
            "/api/resume/{}/portfolio_projects/{}/technologies",
            resume_id, project_id
        ),
        serde_json::json!({ "technology_name": "Diesel", "display_order": 1 }),
    );

    let language_id = post_child(
        fixture,
        format!("/api/resume/{}/languages", resume_id),
        serde_json::json!({ "language_name": "Rust", "display_order": 1 }),
    );

    let framework_id = post_child(
        fixture,
        format!(
            "/api/resume/{}/languages/{}/frameworks",
            resume_id, language_id
        ),
        serde_json::json!({ "framework_name": "Rocket", "display_order": 1 }),
    );

    ChildIds {
        education_id,
        education_key_point_id,
        skill_id,
        work_id,
        work_key_point_id,
        project_id,
        project_key_point_id,
        technology_id,
        language_id,
        framework_id,
    }
}

struct ChildIds {
    education_id: i64,
    education_key_point_id: i64,
    skill_id: i64,
    work_id: i64,
    work_key_point_id: i64,
    project_id: i64,
    project_key_point_id: i64,
    technology_id: i64,
    language_id: i64,
    framework_id: i64,
}

fn keyed(snapshot: &Value, collection: &str, key: i64) -> Value {
    snapshot[collection][key.to_string()].clone()
}

fn restore_revision(
    fixture: &support::Fixture,
    resume_id: Uuid,
    revision: i64,
    expected_revision: i64,
) -> (Status, Value) {
    post_json(
        fixture,
        format!("/api/resume/{}/revisions/{}/restore", resume_id, revision),
        serde_json::json!({ "expected_revision": expected_revision }),
    )
}

#[test]
fn snapshot_omits_identity_and_lifecycle_fields() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 5);
    let resume_id = create_resume(&fixture, "Shape", "Here");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "There" })
        ),
        Status::Ok
    );

    let (_, revision) = fetch_revision(&fixture, resume_id, 1);
    let snapshot = &revision["body"]["snapshot"];

    for field in [
        "id",
        "created_at",
        "updated_at",
        "created_by",
        "base_resume_id",
    ] {
        assert!(
            snapshot.get(field).is_none(),
            "snapshot must not carry {}",
            field
        );
    }

    assert_eq!(snapshot["schema_version"].as_i64(), Some(1));
    assert_eq!(snapshot["show_variant_tag"].as_bool(), Some(true));
    assert_eq!(snapshot["visibility"].as_str(), Some("private"));
    assert!(snapshot["name"].is_string());
    assert!(snapshot["education"].is_array());
}

#[test]
fn snapshot_carries_every_child_collection() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 6);
    let resume_id = create_resume(&fixture, "All Children", "Here");
    fixture.track_resume_id(resume_id);

    let ids = create_child_set(&fixture, resume_id);

    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "Moved" })
        ),
        Status::Ok
    );

    let (status, timeline) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    let head = timeline["body"][0]["revision"]
        .as_i64()
        .expect("the edit just captured a revision");

    let (status, revision) = fetch_revision(&fixture, resume_id, head);
    assert_eq!(status, Status::Ok);
    let snapshot = &revision["body"]["snapshot"];

    assert_eq!(
        snapshot["education"][0]["id"].as_i64(),
        Some(ids.education_id)
    );
    assert_eq!(
        keyed(snapshot, "education_key_points", ids.education_id)[0]["id"].as_i64(),
        Some(ids.education_key_point_id)
    );
    assert_eq!(snapshot["skills"][0]["id"].as_i64(), Some(ids.skill_id));
    assert_eq!(
        snapshot["work_experiences"][0]["id"].as_i64(),
        Some(ids.work_id)
    );
    assert_eq!(
        keyed(snapshot, "work_experience_key_points", ids.work_id)[0]["id"].as_i64(),
        Some(ids.work_key_point_id)
    );
    assert_eq!(
        snapshot["portfolio_projects"][0]["id"].as_i64(),
        Some(ids.project_id)
    );
    assert_eq!(
        keyed(snapshot, "portfolio_key_points", ids.project_id)[0]["id"].as_i64(),
        Some(ids.project_key_point_id)
    );
    assert_eq!(
        keyed(snapshot, "portfolio_technologies", ids.project_id)[0]["id"].as_i64(),
        Some(ids.technology_id)
    );
    assert_eq!(
        snapshot["languages"][0]["id"].as_i64(),
        Some(ids.language_id)
    );
    assert_eq!(
        keyed(snapshot, "frameworks", ids.language_id)[0]["id"].as_i64(),
        Some(ids.framework_id)
    );
}

#[test]
fn markdown_import_writes_one_revision() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 7);
    let resume_id = create_resume(&fixture, "Import Subject", "Original");
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

    let markdown = format!(
        "# Import Subject\n\n- Location: Imported\n- Email: {}\n\n## Skills\n\n- Rust - 90%\n",
        support::unique_email("revision.import")
    );

    let response = fixture
        .client()
        .post(format!("/api/resume/{}/import/markdown", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::Markdown)
        .body(markdown)
        .dispatch();
    assert_eq!(
        response.status(),
        Status::Ok,
        "import into an existing resume"
    );

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        revision_numbers(&body),
        vec![1],
        "a whole-aggregate replace is a single revision"
    );

    let (_, revision) = fetch_revision(&fixture, resume_id, 1);
    let snapshot = &revision["body"]["snapshot"];
    assert_eq!(
        snapshot["email"].as_str(),
        Some(original_email.as_str()),
        "the revision holds the state from before the import"
    );
    assert_eq!(
        snapshot["education"]
            .as_array()
            .expect("education array")
            .len(),
        0
    );
}

#[test]
fn restore_reapplies_content_and_appends_a_revision() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 8);
    let resume_id = create_resume(&fixture, "Restore Subject", "L0");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );
    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L2" })),
        Status::Ok
    );

    let (status, restored) = restore_revision(&fixture, resume_id, 1, 2);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        restored["body"]["location"].as_str(),
        Some("L0"),
        "restoring revision 1 applies the state from before the first edit"
    );

    let (_, body) = list_revisions(&fixture, resume_id);
    assert_eq!(
        revision_numbers(&body),
        vec![3, 2, 1],
        "a restore appends rather than truncating"
    );

    let head = &body["body"][0];
    assert_eq!(head["is_restore"].as_bool(), Some(true));
    assert_eq!(head["action"].as_str(), Some("personalinfo"));

    let (_, appended) = fetch_revision(&fixture, resume_id, 3);
    assert_eq!(
        appended["body"]["snapshot"]["location"].as_str(),
        Some("L2"),
        "the appended revision holds the state the restore replaced"
    );

    let (_, current) = get_json(
        fixture.client(),
        format!("/api/resume/{}", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(current["body"]["location"].as_str(), Some("L0"));
}

#[test]
fn restore_preserves_display_order_and_active() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 9);
    let resume_id = create_resume(&fixture, "Ordering", "Here");
    fixture.track_resume_id(resume_id);

    let education_id = create_education(&fixture, resume_id);

    assert_eq!(
        put_education(
            &fixture,
            education_id,
            serde_json::json!({ "display_order": 9, "active": false })
        ),
        Status::Ok
    );

    let (status, _) = restore_revision(&fixture, resume_id, 2, 2);
    assert_eq!(status, Status::Ok);

    let (status, educations) = get_json(
        fixture.client(),
        format!("/api/resume/{}/education", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(status, Status::Ok);
    let restored = &educations["body"][0];
    assert_eq!(restored["display_order"].as_i64(), Some(1));
    assert_eq!(restored["active"].as_bool(), Some(true));
}

#[test]
fn restore_leaves_identity_and_variant_email_alone() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 10);
    let base_id = create_resume(&fixture, "Variant Base", "Base Place");
    fixture.track_resume_id(base_id);

    let (_, base) = get_json(
        fixture.client(),
        format!("/api/resume/{}", base_id),
        fixture.auth_header(),
    );
    let base_email = base["body"]["email"]
        .as_str()
        .expect("base email")
        .to_string();

    let (status, variant) = post_json(
        &fixture,
        format!("/api/resume/{}/variants", base_id),
        serde_json::json!({ "variant_label": "Acme" }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = support::resume_uuid(&variant["body"]["id"]);
    fixture.track_resume_id(variant_id);

    assert_eq!(
        put_resume(
            &fixture,
            variant_id,
            serde_json::json!({ "location": "Variant Place" })
        ),
        Status::Ok
    );

    let (status, _) = restore_revision(&fixture, variant_id, 1, 1);
    assert_eq!(status, Status::Ok);

    let (status, current) = get_json(
        fixture.client(),
        format!("/api/resume/{}", variant_id),
        fixture.auth_header(),
    );
    assert_eq!(status, Status::Ok);
    assert_eq!(current["body"]["email"].as_str(), Some(base_email.as_str()));
    assert_eq!(
        support::resume_uuid(&current["body"]["id"]),
        variant_id,
        "a restore does not rewrite row identity"
    );
    assert_eq!(
        support::resume_uuid(&current["body"]["base_resume_id"]),
        base_id
    );
    assert_eq!(
        current["body"]["created_by"].as_i64(),
        Some(fixture.user_id() as i64)
    );
}

#[test]
fn restore_restores_visibility() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 11);
    let resume_id = create_resume(&fixture, "Visibility", "Here");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "visibility": "public" })
        ),
        Status::Ok
    );

    let (status, restored) = restore_revision(&fixture, resume_id, 1, 1);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        restored["body"]["visibility"].as_str(),
        Some("private"),
        "visibility is content, so a restore brings it back"
    );
}

#[test]
fn diff_path_is_a_json_pointer() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 12);
    let resume_id = create_resume(&fixture, "Diff Subject", "Before");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "After" })
        ),
        Status::Ok
    );

    let (status, diff) = get_json(
        fixture.client(),
        format!("/api/resume/{}/revisions/1/diff", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(status, Status::Ok);

    let changes = diff["body"]["changes"].as_array().expect("changes array");
    let location = changes
        .iter()
        .find(|change| change["path"].as_str() == Some("/location"))
        .expect("a change for /location");

    assert_eq!(location["kind"].as_str(), Some("changed"));
    assert_eq!(location["before"].as_str(), Some("Before"));
    assert_eq!(location["after"].as_str(), Some("After"));
}

#[test]
fn older_snapshot_missing_a_field_deserializes() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 13);
    let resume_id = create_resume(&fixture, "Old Snapshot", "Here");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(
            &fixture,
            resume_id,
            serde_json::json!({ "location": "Moved" })
        ),
        Status::Ok
    );

    let mut connection = infrastructure::establish_connection();
    diesel::sql_query(
        "UPDATE resume_revisions SET snapshot = snapshot - 'video' WHERE resume_id = $1 AND revision = 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(resume_id)
    .execute(&mut connection)
    .expect("strip a field from the stored snapshot");

    let (status, revision) = fetch_revision(&fixture, resume_id, 1);
    assert_eq!(status, Status::Ok, "a snapshot missing a field still reads");
    assert_eq!(
        revision["body"]["snapshot"]["video"],
        Value::Null,
        "the missing field reads as absent"
    );
}

#[test]
fn restore_with_stale_expected_head_is_409() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 14);
    let resume_id = create_resume(&fixture, "Stale Guard", "L0");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );
    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L2" })),
        Status::Ok
    );

    let (status, _) = restore_revision(&fixture, resume_id, 1, 1);
    assert_eq!(
        status,
        Status::Conflict,
        "a client holding a stale head is refused"
    );

    let (_, body) = list_revisions(&fixture, resume_id);
    assert_eq!(
        revision_numbers(&body),
        vec![2, 1],
        "a refused restore writes nothing"
    );

    let (_, current) = get_json(
        fixture.client(),
        format!("/api/resume/{}", resume_id),
        fixture.auth_header(),
    );
    assert_eq!(current["body"]["location"].as_str(), Some("L2"));

    let (status, _) = restore_revision(&fixture, resume_id, 1, 2);
    assert_eq!(status, Status::Ok, "the current head is accepted");
}

#[test]
fn restore_publishes_personalinfo_for_the_target_resume() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 15);
    let resume_id = create_resume(&fixture, "Publish", "L0");
    fixture.track_resume_id(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );

    let mut receiver = fixture.hub.subscribe(resume_id);

    let (status, _) = restore_revision(&fixture, resume_id, 1, 1);
    assert_eq!(status, Status::Ok);

    let event = receiver.try_recv().expect("a restore notifies subscribers");
    assert_eq!(event.resume_id, resume_id);
    assert_eq!(
        event.action,
        api::realtime::ResumeChangedAction::Updated(api::realtime::SectionType::PersonalInfo)
    );
    assert!(
        receiver.try_recv().is_err(),
        "a restore publishes exactly one event"
    );
}

#[test]
fn capture_publishes_nothing_extra() {
    let mut fixture = support::Fixture::new(LOCK_KEY + 16);
    let resume_id = create_resume(&fixture, "Quiet Capture", "L0");
    fixture.track_resume_id(resume_id);

    let mut receiver = fixture.hub.subscribe(resume_id);

    assert_eq!(
        put_resume(&fixture, resume_id, serde_json::json!({ "location": "L1" })),
        Status::Ok
    );

    let event = receiver
        .try_recv()
        .expect("the mutation notifies subscribers");
    assert_eq!(
        event.action,
        api::realtime::ResumeChangedAction::Updated(api::realtime::SectionType::PersonalInfo)
    );
    assert!(
        receiver.try_recv().is_err(),
        "capturing the pre-state publishes nothing of its own"
    );
}

#[test]
fn concurrent_edits_serialize_without_losing_a_revision() {
    struct NullPublisher;

    impl application::realtime::ResumeChangePublisher for NullPublisher {
        fn resume_changed(
            &self,
            _resume_id: Uuid,
            _action: application::realtime::ResumeChangedAction,
        ) {
        }
    }

    let mut fixture = support::Fixture::new(LOCK_KEY + 17);
    let resume_id = create_resume(&fixture, "Concurrent", "L0");
    fixture.track_resume_id(resume_id);
    let user_id = fixture.user_id();

    let handles: Vec<_> = ["A", "B"]
        .into_iter()
        .map(|location| {
            std::thread::spawn(move || {
                let dto: domain::models::UpdateResume =
                    serde_json::from_value(serde_json::json!({ "location": location }))
                        .expect("update dto");

                application::resume::update::update_resume(user_id, resume_id, dto, &NullPublisher)
                    .expect("a concurrent edit serializes instead of failing")
                    .location
            })
        })
        .collect();

    let mut written: Vec<String> = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread joins").expect("location"))
        .collect();
    written.sort();

    let (status, body) = list_revisions(&fixture, resume_id);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        revision_numbers(&body),
        vec![2, 1],
        "both edits are recorded, with no gap and no lost write"
    );

    let (_, first) = fetch_revision(&fixture, resume_id, 1);
    assert_eq!(
        first["body"]["snapshot"]["location"].as_str(),
        Some("L0"),
        "the first revision still holds the original state"
    );

    let (_, second) = fetch_revision(&fixture, resume_id, 2);
    let second_pre = second["body"]["snapshot"]["location"]
        .as_str()
        .expect("pre-state")
        .to_string();
    assert!(
        second_pre == "A" || second_pre == "B",
        "the second edit observes the first one's committed write"
    );

    let (_, current) = get_json(
        fixture.client(),
        format!("/api/resume/{}", resume_id),
        fixture.auth_header(),
    );
    let live = current["body"]["location"]
        .as_str()
        .expect("live location")
        .to_string();

    assert_eq!(written, vec!["A".to_string(), "B".to_string()]);
    assert_ne!(
        live, second_pre,
        "the live row is the last writer's state, never a recorded pre-image"
    );
}
