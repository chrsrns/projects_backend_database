use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn markdown(name: &str, email: &str) -> String {
    format!(
        "# {}\n\n- Location: New York, NY\n- Email: {}\n- Public: true\n\n## Skills\n\n- Rust - 90%\n",
        name, email
    )
}

fn import_markdown(fixture: &support::Fixture, name: &str, email: &str) -> (Status, Value) {
    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(markdown(name, email))
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn post_variant(fixture: &support::Fixture, base_id: i32, body: Value) -> (Status, Value) {
    let response = fixture
        .client()
        .post(format!("/api/resume/{}/variants", base_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn put_resume(fixture: &support::Fixture, resume_id: i32, body: Value) -> (Status, Value) {
    let response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn get_resume(fixture: &support::Fixture, resume_id: i32) -> Value {
    let response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    serde_json::from_str(&response.into_string().unwrap()).unwrap()
}

#[test]
fn variant_metadata_absent_from_markdown_and_resume_document() {
    let mut fixture = support::Fixture::new(9_241_041);
    let email = format!(
        "variant.md.meta.{}.{}@example.com",
        9_241_041,
        unique_suffix()
    );

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let metadata = [
        "Acme Corporation",
        "Staff Backend Engineer",
        "Acme tailoring",
        "Own the resume pipeline",
        "2026-03",
    ];

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({
            "company_name": metadata[0],
            "role_title": metadata[1],
            "variant_label": metadata[2],
            "job_description": metadata[3],
            "target_date": metadata[4]
        }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let export = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", variant_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export.status(), Status::Ok);
    let exported = export.into_string().expect("markdown body");

    assert!(
        exported.contains("Jane Doe"),
        "resume content still exports"
    );
    for value in metadata {
        assert!(
            !exported.contains(value),
            "variant metadata must not reach Markdown: {}",
            value
        );
    }

    let convert = fixture
        .client()
        .post("/api/resume/convert/markdown")
        .header(ContentType::new("text", "markdown"))
        .body(exported)
        .dispatch();
    assert_eq!(convert.status(), Status::Ok);
    let converted: Value = serde_json::from_str(&convert.into_string().unwrap()).unwrap();

    assert_eq!(
        converted["body"]["schema_version"], 1,
        "the document contract version is unchanged"
    );

    let document = serde_json::to_string(&converted["body"]["document"]).unwrap();
    for value in metadata {
        assert!(
            !document.contains(value),
            "variant metadata must not reach ResumeDocument: {}",
            value
        );
    }
}

#[test]
fn variant_markdown_export_reimport_updates_base() {
    let mut fixture = support::Fixture::new(9_241_042);
    let email = format!(
        "variant.md.reimport.{}.{}@example.com",
        9_241_042,
        unique_suffix()
    );

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "company_name": "Acme Corp" }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let (status, _) = put_resume(
        &fixture,
        variant_id,
        serde_json::json!({ "name": "Acme Tailored" }),
    );
    assert_eq!(status, Status::Ok);

    let export = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", variant_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export.status(), Status::Ok);
    let exported = export.into_string().expect("markdown body");
    assert!(exported.contains("Acme Tailored"));
    assert!(
        !exported.contains("Acme Corp"),
        "targeting metadata stays out of the export"
    );

    let reimport = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(exported)
        .dispatch();
    assert_eq!(
        reimport.status(),
        Status::Ok,
        "the base is updated in place"
    );

    let reimported: Value = serde_json::from_str(&reimport.into_string().unwrap()).unwrap();
    assert_eq!(
        reimported["body"]["id"].as_i64().unwrap() as i32,
        base_id,
        "a variant's export updates its base, never the variant row"
    );
    assert_eq!(reimported["body"]["name"], "Acme Tailored");
    assert!(reimported["body"]["base_resume_id"].is_null());

    let variant_after = get_resume(&fixture, variant_id);
    assert_eq!(
        variant_after["body"]["base_resume_id"].as_i64().unwrap() as i32,
        base_id,
        "the variant stays linked to its base"
    );
    assert_eq!(variant_after["body"]["company_name"], "Acme Corp");
}

#[test]
fn markdown_import_never_updates_variant_row() {
    let mut fixture = support::Fixture::new(9_241_040);
    let email = format!("variant.md.{}.{}@example.com", 9_241_040, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "company_name": "Acme Corp" }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);
    assert_eq!(
        variant["body"]["email"], email,
        "the variant carries its base email"
    );

    let (status, _) = put_resume(
        &fixture,
        variant_id,
        serde_json::json!({ "name": "Tailored Copy" }),
    );
    assert_eq!(status, Status::Ok);

    // The same email now matches both rows; the import must pick the base.
    let (status, updated) = import_markdown(&fixture, "Jane Doe Updated", &email);
    assert_eq!(status, Status::Ok, "the base is updated, not created");
    assert_eq!(
        updated["body"]["id"].as_i64().unwrap() as i32,
        base_id,
        "the import target is the base resume"
    );
    assert_eq!(updated["body"]["name"], "Jane Doe Updated");

    let variant_after = get_resume(&fixture, variant_id);
    assert_eq!(
        variant_after["body"]["name"], "Tailored Copy",
        "the variant keeps its own content"
    );
    assert_eq!(variant_after["body"]["company_name"], "Acme Corp");
    assert_eq!(
        variant_after["body"]["base_resume_id"].as_i64().unwrap() as i32,
        base_id
    );
}
