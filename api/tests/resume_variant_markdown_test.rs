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
fn variant_metadata_in_front_matter_but_not_resume_document() {
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
    assert!(
        exported.starts_with("---\n"),
        "the export opens with a front-matter block"
    );
    for value in metadata {
        assert!(
            exported.contains(value),
            "variant metadata round-trips through front-matter: {}",
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
fn variant_markdown_export_reimport_updates_variant() {
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
        exported.contains("company_name: \"Acme Corp\""),
        "the export carries the variant's targeting metadata"
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
        "the variant is updated in place"
    );

    let reimported: Value = serde_json::from_str(&reimport.into_string().unwrap()).unwrap();
    assert_eq!(
        reimported["body"]["id"].as_i64().unwrap() as i32,
        variant_id,
        "the marker routes the import back to the variant row"
    );
    assert_eq!(reimported["body"]["name"], "Acme Tailored");
    assert_eq!(reimported["body"]["is_variant"], true);
    assert_eq!(
        reimported["body"]["base_resume_id"].as_i64().unwrap() as i32,
        base_id,
        "the variant stays linked to its base"
    );
    assert_eq!(reimported["body"]["company_name"], "Acme Corp");
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

fn import_markdown_into(
    fixture: &support::Fixture,
    resume_id: i32,
    body: String,
) -> (Status, Value) {
    let response = fixture
        .client()
        .post(format!("/api/resume/{}/import/markdown", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(body)
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

#[test]
fn explicit_route_updates_named_base() {
    let mut fixture = support::Fixture::new(9_241_055);
    let email = format!("explicit.base.{}.{}@example.com", 9_241_055, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, updated) =
        import_markdown_into(&fixture, base_id, markdown("Renamed", "renamed@example.com"));
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["id"], base_id);
    assert_eq!(updated["body"]["name"], "Renamed");
}

#[test]
fn explicit_route_absent_returns_404() {
    let fixture = support::Fixture::new(9_241_056);
    let (status, _) =
        import_markdown_into(&fixture, 999_999_999, markdown("Jane Doe", "x@example.com"));
    assert_eq!(status, Status::NotFound);
}

#[test]
fn explicit_route_foreign_returns_403() {
    let mut fixture = support::Fixture::new(9_241_057);
    let email = format!("explicit.f.{}.{}@example.com", 9_241_057, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let other = support::register_and_login(fixture.client(), "explicit.other");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    let response = fixture
        .client()
        .post(format!("/api/resume/{}/import/markdown", base_id))
        .header(support::auth_header(&other.token))
        .header(ContentType::new("text", "markdown"))
        .body(markdown("Not Yours", "y@example.com"))
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
fn explicit_route_rejects_conflicting_marker() {
    let mut fixture = support::Fixture::new(9_241_058);
    let email_a = format!("explicit.a.{}.{}@example.com", 9_241_058, unique_suffix());
    let email_b = format!("explicit.b.{}.{}@example.com", 9_241_058, unique_suffix());

    let (status, a) = import_markdown(&fixture, "Resume A", &email_a);
    assert_eq!(status, Status::Created);
    let a_id = a["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(a_id);

    let (status, b) = import_markdown(&fixture, "Resume B", &email_b);
    assert_eq!(status, Status::Created);
    let b_id = b["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(b_id);

    let (status, _) = import_markdown_into(
        &fixture,
        a_id,
        marked("Clash", &email_a, &b_id.to_string()),
    );
    assert_eq!(status, Status::BadRequest);

    let (status, updated) =
        import_markdown_into(&fixture, a_id, marked("Agreed", &email_a, &a_id.to_string()));
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["name"], "Agreed");
}

fn export_markdown(fixture: &support::Fixture, resume_id: i32) -> String {
    let response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    response.into_string().expect("markdown body")
}

#[test]
fn variant_marker_reimport_updates_variant() {
    let mut fixture = support::Fixture::new(9_241_060);
    let email = format!("variant.marker.{}.{}@example.com", 9_241_060, unique_suffix());

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

    let exported = export_markdown(&fixture, variant_id);
    assert!(exported.contains("Acme Tailored"));
    assert!(exported.contains(&format!("resume_id: {}", variant_id)));

    let (status, updated) = import_markdown_body(&fixture, exported);
    assert_eq!(status, Status::Ok, "the marker re-imports into the variant");
    assert_eq!(updated["body"]["id"], variant_id);
    assert_eq!(updated["body"]["is_variant"], true);
    assert_eq!(updated["body"]["company_name"], "Acme Corp");

    let base_after = get_resume(&fixture, base_id);
    assert_eq!(
        base_after["body"]["name"], "Jane Doe",
        "the base keeps its own content"
    );
}

#[test]
fn variant_import_keeps_stored_email_and_visibility() {
    let mut fixture = support::Fixture::new(9_241_061);
    let email = format!("variant.keep.{}.{}@example.com", 9_241_061, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "is_public": false }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);
    assert_eq!(variant["body"]["is_public"], false);

    // The markdown body demands a different email and Public: true; on a
    // variant target both are ignored and the stored values win.
    let imported = marked("Jane Doe", "totally.different@example.com", &variant_id.to_string())
        .replace("- Public: true", "- Public: false");
    let (status, updated) = import_markdown_body(&fixture, imported);
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["email"], email);
    assert_eq!(updated["body"]["is_public"], false);
}

#[test]
fn metadata_keys_on_base_target_return_400() {
    let mut fixture = support::Fixture::new(9_241_062);
    let email = format!("meta.base.{}.{}@example.com", 9_241_062, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let body = format!(
        "---\nresume_id: {}\ncompany_name: \"Acme\"\n---\n{}",
        base_id,
        markdown("Jane Doe", &email)
    );
    let (status, _) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::BadRequest);

    let (status, _) = import_markdown_into(
        &fixture,
        base_id,
        format!("---\ncompany_name: \"Acme\"\n---\n{}", markdown("Jane Doe", &email)),
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn metadata_keys_on_create_path_return_400() {
    let fixture = support::Fixture::new(9_241_063);
    let email = format!("meta.create.{}.{}@example.com", 9_241_063, unique_suffix());

    let body = format!(
        "---\ncompany_name: \"Acme\"\n---\n{}",
        markdown("Jane Doe", &email)
    );
    let (status, _) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn variant_metadata_apply_absent_null_and_caps() {
    let mut fixture = support::Fixture::new(9_241_064);
    let email = format!("meta.apply.{}.{}@example.com", 9_241_064, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({
            "company_name": "Acme Corp",
            "role_title": "Engineer",
            "target_date": "2026-03"
        }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    // Absent keys leave stored values; a null key stores NULL; a string
    // applies after validation; a number on a string key is rejected.
    let body = format!(
        "---\nresume_id: {0}\nrole_title: null\nvariant_label: \"tailored\"\nshow_variant_tag: false\n---\n{1}",
        variant_id,
        markdown("Jane Doe", &email)
    );
    let (status, updated) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["company_name"], "Acme Corp");
    assert!(updated["body"]["role_title"].is_null());
    assert_eq!(updated["body"]["variant_label"], "tailored");
    assert_eq!(updated["body"]["show_variant_tag"], false);
    assert_eq!(updated["body"]["target_date"], "2026-03");

    let body = format!(
        "---\nresume_id: {0}\ncompany_name: \"{1}\"\n---\n{2}",
        variant_id,
        "x".repeat(256),
        markdown("Jane Doe", &email)
    );
    let (status, _) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::BadRequest);

    let body = format!(
        "---\nresume_id: {0}\ntarget_date: \"not-a-date\"\n---\n{1}",
        variant_id,
        markdown("Jane Doe", &email)
    );
    let (status, _) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::BadRequest);

    let body = format!(
        "---\nresume_id: {0}\ncompany_name: 42\n---\n{1}",
        variant_id,
        markdown("Jane Doe", &email)
    );
    let (status, _) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::BadRequest);

    let body = format!(
        "---\nresume_id: {0}\nshow_variant_tag: \"yes\"\n---\n{1}",
        variant_id,
        markdown("Jane Doe", &email)
    );
    let (status, _) = import_markdown_body(&fixture, body);
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn stale_email_marker_still_targets_variant() {
    let mut fixture = support::Fixture::new(9_241_065);
    let email = format!("stale.email.{}.{}@example.com", 9_241_065, unique_suffix());

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

    let exported = export_markdown(&fixture, variant_id);

    // The base email moves after the export; the marker must still route the
    // re-import instead of creating a duplicate resume.
    let (status, _) = put_resume(
        &fixture,
        base_id,
        serde_json::json!({ "email": format!("moved.{}@example.com", unique_suffix()) }),
    );
    assert_eq!(status, Status::Ok);

    let (status, updated) = import_markdown_body(&fixture, exported);
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["id"], variant_id);

    let list = fixture
        .client()
        .get("/api/resumes")
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list.status(), Status::Ok);
    let resumes: Value = serde_json::from_str(&list.into_string().unwrap()).unwrap();
    let ids: Vec<i64> = resumes["body"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_i64().unwrap())
        .filter(|id| *id == base_id as i64 || *id == variant_id as i64)
        .collect();
    assert_eq!(ids.len(), 2, "no duplicate resume was created");
}

#[test]
fn malformed_front_matter_returns_400_on_import() {
    let fixture = support::Fixture::new(9_241_066);
    let email = format!("fm.bad.{}.{}@example.com", 9_241_066, unique_suffix());
    let body_md = markdown("Jane Doe", &email);

    for front_matter in [
        "resume_id: 1\nresume_id: 2",  // duplicate key
        "resume_id: 1\n\nrole_title: x", // blank line inside block
        "company_name: 42",            // metadata key without a variant target
        "unknown: 1",                  // unknown key
        "garbage line without colon",  // malformed line
    ] {
        let (status, _) = import_markdown_body(
            &fixture,
            format!("---\n{}\n---\n{}", front_matter, body_md),
        );
        assert_eq!(status, Status::BadRequest, "block: {:?}", front_matter);
    }

    // Unclosed fence.
    let (status, _) =
        import_markdown_body(&fixture, format!("---\nresume_id: 1\n{}", body_md));
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn non_owner_export_omits_show_variant_tag() {
    let mut fixture = support::Fixture::new(9_241_067);
    let email = format!("tag.view.{}.{}@example.com", 9_241_067, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "company_name": "Acme", "show_variant_tag": false }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    // Owner sees the tag flag in the front-matter.
    let owner_export = export_markdown(&fixture, variant_id);
    assert!(owner_export.contains("show_variant_tag: false"));

    // An anonymous viewer of the public variant does not.
    let anon_export = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", variant_id))
        .dispatch();
    assert_eq!(anon_export.status(), Status::Ok);
    let exported = anon_export.into_string().expect("markdown body");
    assert!(exported.contains(&format!("resume_id: {}", variant_id)));
    assert!(exported.contains("company_name: \"Acme\""));
    assert!(
        !exported.contains("show_variant_tag"),
        "the tag flag is owner-only"
    );
}

#[test]
fn variant_import_publishes_variant_id() {
    let mut fixture = support::Fixture::new(9_241_068);
    let email = format!("variant.pub.{}.{}@example.com", 9_241_068, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "company_name": "Acme" }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let mut variant_rx = fixture.hub.subscribe(variant_id);
    let mut base_rx = fixture.hub.subscribe(base_id);

    let imported = marked("Jane Doe", &email, &variant_id.to_string());
    let (status, _) = import_markdown_body(&fixture, imported);
    assert_eq!(status, Status::Ok);

    let evt = variant_rx.try_recv().expect("variant changed event");
    assert_eq!(evt.resume_id, variant_id);
    assert_eq!(
        evt.action,
        api::realtime::ResumeChangedAction::Updated(api::realtime::SectionType::PersonalInfo)
    );
    assert!(
        base_rx.try_recv().is_err(),
        "subscribers of the base are not notified"
    );
}

#[test]
fn explicit_route_targets_variant() {
    let mut fixture = support::Fixture::new(9_241_069);
    let email = format!("explicit.var.{}.{}@example.com", 9_241_069, unique_suffix());

    let (status, base) = import_markdown(&fixture, "Jane Doe", &email);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let (status, variant) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "company_name": "Acme" }),
    );
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let body = format!(
        "---\nresume_id: {0}\nvariant_label: \"explicit\"\n---\n{1}",
        variant_id,
        markdown("Variant Renamed", &email)
    );
    let (status, updated) = import_markdown_into(&fixture, variant_id, body);
    assert_eq!(status, Status::Ok);
    assert_eq!(updated["body"]["id"], variant_id);
    assert_eq!(updated["body"]["name"], "Variant Renamed");
    assert_eq!(updated["body"]["variant_label"], "explicit");
    assert_eq!(updated["body"]["is_variant"], true);

    let base_after = get_resume(&fixture, base_id);
    assert_eq!(base_after["body"]["name"], "Jane Doe");
}
