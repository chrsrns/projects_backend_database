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

fn import_markdown_body(fixture: &support::Fixture, body: String) -> (Status, Value) {
    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(body)
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn import_markdown_body_as(
    fixture: &support::Fixture,
    token: &str,
    body: String,
) -> (Status, Value) {
    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(support::auth_header(token))
        .header(ContentType::new("text", "markdown"))
        .body(body)
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn marked(name: &str, email: &str, resume_id: &str) -> String {
    format!("---\nresume_id: {}\n---\n{}", resume_id, markdown(name, email))
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

#[test]
fn marker_targets_named_base_row() {
    let mut fixture = support::Fixture::new(9_241_050);
    let email = format!("marker.base.{}.{}@example.com", 9_241_050, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let fresh_email = format!(
        "marker.base.new.{}.{}@example.com",
        9_241_050,
        unique_suffix()
    );
    let (status, updated) =
        import_markdown_body(&fixture, marked("Jane Marker", &fresh_email, &base_id.to_string()));
    assert_eq!(status, Status::Ok, "the marker resolves the row");
    assert_eq!(updated["body"]["id"], base_id);
    assert_eq!(updated["body"]["name"], "Jane Marker");
    assert_eq!(updated["body"]["email"], fresh_email);
}

#[test]
fn marker_absent_row_returns_404() {
    let fixture = support::Fixture::new(9_241_051);
    let email = format!("marker.miss.{}.{}@example.com", 9_241_051, unique_suffix());

    let (status, _) =
        import_markdown_body(&fixture, marked("Jane Doe", &email, "999999999"));
    assert_eq!(status, Status::NotFound);
}

#[test]
fn marker_foreign_row_returns_403() {
    let mut fixture = support::Fixture::new(9_241_052);
    let email = format!("marker.foreign.{}.{}@example.com", 9_241_052, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let other = support::register_and_login(fixture.client(), "marker.foreign.other");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    let other_email = format!(
        "marker.foreign.other.{}.{}@example.com",
        9_241_052,
        unique_suffix()
    );
    let (status, _) = import_markdown_body_as(
        &fixture,
        &other.token,
        marked("Not Yours", &other_email, &base_id.to_string()),
    );
    assert_eq!(
        status,
        Status::Forbidden,
        "a marker never falls back to email matching"
    );
}

#[test]
fn marker_malformed_values_return_400() {
    let fixture = support::Fixture::new(9_241_053);
    let email = format!("marker.bad.{}.{}@example.com", 9_241_053, unique_suffix());

    for resume_id in ["\"abc\"", "0", "-3", "\"\"", "", "true", "1.5"] {
        let (status, _) = import_markdown_body(
            &fixture,
            marked("Jane Doe", &format!("{}.{}", email, resume_id), resume_id),
        );
        assert_eq!(status, Status::BadRequest, "resume_id value: {}", resume_id);
    }
}

#[test]
fn marker_wins_over_email_match() {
    let mut fixture = support::Fixture::new(9_241_054);
    let email_a = format!("marker.wins.a.{}.{}@example.com", 9_241_054, unique_suffix());
    let email_b = format!("marker.wins.b.{}.{}@example.com", 9_241_054, unique_suffix());

    let (status, a) = import_markdown(&fixture, "Resume A", &email_a);
    assert_eq!(status, Status::Created);
    let a_id = a["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(a_id);

    let (status, b) = import_markdown(&fixture, "Resume B", &email_b);
    assert_eq!(status, Status::Created);
    let b_id = b["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(b_id);

    // Marker names A while the Email bullet matches B: the marker wins and B
    // is left alone. A's email becomes the B email (still unique).
    let email_new = format!(
        "marker.wins.new.{}.{}@example.com",
        9_241_054,
        unique_suffix()
    );
    let (status, updated) =
        import_markdown_body(&fixture, marked("A via marker", &email_new, &a_id.to_string()));
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["id"], a_id);
    assert_eq!(updated["body"]["name"], "A via marker");

    let b_after = get_resume(&fixture, b_id);
    assert_eq!(b_after["body"]["name"], "Resume B");
    assert_eq!(b_after["body"]["email"], email_b);
}
