use application::error::ApplicationError;
use application::resume::variant::{create_variant, list_variants};
use domain::models::{NewVariantRequest, PartialDate};
use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

const VOLATILE: [&str; 4] = ["id", "resume_id", "created_at", "updated_at"];

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn variant_markdown(email: &str, public: bool) -> String {
    format!(
        "# Jane Doe\n\n- Location: New York, NY\n- Email: {}\n- GitHub: https://github.com/janedoe\n- Mobile: +1987654321\n- Public: {}\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024)\n- Degree: Bachelor of Science\n- Description: Focused on software engineering\n- Graduated with honors\n- Specialized in distributed systems\n\n## Skills\n\n- Rust - 90%\n- Python - 75%\n- JavaScript - 60%\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (Jan 2020 - Present)\n- Description: Backend development\n- Led team of 5 developers\n- Improved system performance by 40%\n\n## Portfolio Projects\n\n### My Portfolio\n- Live: https://example.com\n- Source: https://github.com/janedoe/project\n- Technologies: Rust, Rocket, Diesel\n- Built a scalable resume API\n\n## Languages & Frameworks\n\n### Rust\n- Rocket\n- Actix\n\n### Python\n- Django\n- FastAPI\n",
        email, public
    )
}

fn import_base(fixture: &mut support::Fixture, lock_key: i64) -> i32 {
    import_base_with_visibility(fixture, lock_key, true)
}

fn import_base_with_visibility(fixture: &mut support::Fixture, lock_key: i64, public: bool) -> i32 {
    let email = format!("variant.base.{}.{}@example.com", lock_key, unique_suffix());
    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(variant_markdown(&email, public))
        .dispatch();

    assert_eq!(response.status(), Status::Created, "base import");
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let id = json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(id);
    id
}

fn get_json(fixture: &support::Fixture, path: &str) -> Value {
    let response = fixture
        .client()
        .get(path)
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(response.status(), Status::Ok, "GET {}", path);
    serde_json::from_str(&response.into_string().unwrap()).unwrap()
}

fn strip(value: &Value, keys: &[&str]) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| !keys.contains(&key.as_str()))
                .map(|(key, nested)| (key.clone(), strip(nested, keys)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(|item| strip(item, keys)).collect()),
        other => other.clone(),
    }
}

fn clone_request() -> NewVariantRequest {
    NewVariantRequest {
        company_name: Some("Acme Corp".to_string()),
        role_title: Some("Backend Engineer".to_string()),
        target_date: Some(PartialDate::from_iso_str("2026-03").unwrap()),
        job_description: Some("Build the thing".to_string()),
        variant_label: Some("Acme 2026".to_string()),
        is_public: None,
        show_variant_tag: None,
    }
}

#[test]
fn variant_clone_deep_copies_all_children() {
    let mut fixture = support::Fixture::new(9_241_001);
    let base_id = import_base(&mut fixture, 9_241_001);

    let variant = create_variant(fixture.user_id(), base_id, clone_request()).expect("clone");
    fixture.track_resume_id(variant.id);

    let base = get_json(&fixture, &format!("/api/resume/{}", base_id));
    let cloned = get_json(&fixture, &format!("/api/resume/{}", variant.id));

    let base_resume = strip(&base["body"], &VOLATILE);
    let cloned_resume = strip(&cloned["body"], &VOLATILE);
    assert_eq!(
        base_resume["name"], cloned_resume["name"],
        "clone keeps the base name"
    );
    assert_eq!(
        base_resume["email"], cloned_resume["email"],
        "clone copies the base email"
    );
    assert_eq!(
        base_resume["executive_summary"],
        cloned_resume["executive_summary"]
    );
    assert_eq!(base_resume["is_public"], cloned_resume["is_public"]);

    let base_education = get_json(&fixture, &format!("/api/resume/{}/education", base_id));
    let cloned_education = get_json(&fixture, &format!("/api/resume/{}/education", variant.id));
    let base_education_items = base_education["body"].as_array().unwrap();
    let cloned_education_items = cloned_education["body"].as_array().unwrap();
    assert_eq!(base_education_items.len(), 1);
    assert_eq!(
        strip(&base_education["body"], &VOLATILE),
        strip(&cloned_education["body"], &VOLATILE),
        "education rows are copied verbatim"
    );

    let base_education_id = base_education_items[0]["id"].as_i64().unwrap();
    let cloned_education_id = cloned_education_items[0]["id"].as_i64().unwrap();
    assert_ne!(base_education_id, cloned_education_id, "clone gets new ids");

    let base_key_points = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/education/{}/key_points",
            base_id, base_education_id
        ),
    );
    let cloned_key_points = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/education/{}/key_points",
            variant.id, cloned_education_id
        ),
    );
    assert_eq!(
        strip(
            &base_key_points["body"],
            &["id", "education_id", "created_at", "updated_at"]
        ),
        strip(
            &cloned_key_points["body"],
            &["id", "education_id", "created_at", "updated_at"]
        ),
        "education key points are copied verbatim"
    );

    let base_skills = get_json(&fixture, &format!("/api/resume/{}/skills", base_id));
    let cloned_skills = get_json(&fixture, &format!("/api/resume/{}/skills", variant.id));
    assert_eq!(base_skills["body"].as_array().unwrap().len(), 3);
    assert_eq!(
        strip(&base_skills["body"], &VOLATILE),
        strip(&cloned_skills["body"], &VOLATILE),
        "skills are copied verbatim"
    );

    let base_work = get_json(
        &fixture,
        &format!("/api/resume/{}/work_experiences", base_id),
    );
    let cloned_work = get_json(
        &fixture,
        &format!("/api/resume/{}/work_experiences", variant.id),
    );
    assert_eq!(
        strip(&base_work["body"], &VOLATILE),
        strip(&cloned_work["body"], &VOLATILE),
        "work experiences are copied verbatim"
    );

    let base_work_id = base_work["body"][0]["id"].as_i64().unwrap();
    let cloned_work_id = cloned_work["body"][0]["id"].as_i64().unwrap();
    let base_work_key_points = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            base_id, base_work_id
        ),
    );
    let cloned_work_key_points = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            variant.id, cloned_work_id
        ),
    );
    assert_eq!(
        strip(
            &base_work_key_points["body"],
            &["id", "work_experience_id", "created_at", "updated_at"]
        ),
        strip(
            &cloned_work_key_points["body"],
            &["id", "work_experience_id", "created_at", "updated_at"]
        ),
        "work experience key points are copied verbatim"
    );

    let base_projects = get_json(
        &fixture,
        &format!("/api/resume/{}/portfolio_projects", base_id),
    );
    let cloned_projects = get_json(
        &fixture,
        &format!("/api/resume/{}/portfolio_projects", variant.id),
    );
    assert_eq!(
        strip(&base_projects["body"], &VOLATILE),
        strip(&cloned_projects["body"], &VOLATILE),
        "portfolio projects are copied verbatim"
    );

    let base_project_id = base_projects["body"][0]["id"].as_i64().unwrap();
    let cloned_project_id = cloned_projects["body"][0]["id"].as_i64().unwrap();
    let base_project_key_points = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/portfolio_projects/{}/key_points",
            base_id, base_project_id
        ),
    );
    let cloned_project_key_points = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/portfolio_projects/{}/key_points",
            variant.id, cloned_project_id
        ),
    );
    assert_eq!(
        strip(
            &base_project_key_points["body"],
            &["id", "portfolio_project_id", "created_at", "updated_at"]
        ),
        strip(
            &cloned_project_key_points["body"],
            &["id", "portfolio_project_id", "created_at", "updated_at"]
        ),
        "portfolio key points are copied verbatim"
    );

    let base_technologies = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/portfolio_projects/{}/technologies",
            base_id, base_project_id
        ),
    );
    let cloned_technologies = get_json(
        &fixture,
        &format!(
            "/api/resume/{}/portfolio_projects/{}/technologies",
            variant.id, cloned_project_id
        ),
    );
    assert_eq!(
        strip(
            &base_technologies["body"],
            &["id", "portfolio_project_id", "created_at", "updated_at"]
        ),
        strip(
            &cloned_technologies["body"],
            &["id", "portfolio_project_id", "created_at", "updated_at"]
        ),
        "portfolio technologies are copied verbatim"
    );

    let base_languages = get_json(&fixture, &format!("/api/resume/{}/languages", base_id));
    let cloned_languages = get_json(&fixture, &format!("/api/resume/{}/languages", variant.id));
    assert_eq!(base_languages["body"].as_array().unwrap().len(), 2);
    assert_eq!(
        strip(&base_languages["body"], &VOLATILE),
        strip(&cloned_languages["body"], &VOLATILE),
        "languages are copied verbatim"
    );

    for index in 0..2 {
        let base_language_id = base_languages["body"][index]["id"].as_i64().unwrap();
        let cloned_language_id = cloned_languages["body"][index]["id"].as_i64().unwrap();
        let base_frameworks = get_json(
            &fixture,
            &format!(
                "/api/resume/{}/languages/{}/frameworks",
                base_id, base_language_id
            ),
        );
        let cloned_frameworks = get_json(
            &fixture,
            &format!(
                "/api/resume/{}/languages/{}/frameworks",
                variant.id, cloned_language_id
            ),
        );
        assert_eq!(
            strip(
                &base_frameworks["body"],
                &["id", "language_id", "created_at", "updated_at"]
            ),
            strip(
                &cloned_frameworks["body"],
                &["id", "language_id", "created_at", "updated_at"]
            ),
            "frameworks are copied verbatim"
        );
    }
}

#[test]
fn variant_clone_preserves_child_display_order_and_active() {
    let mut fixture = support::Fixture::new(9_241_002);
    let base_id = import_base(&mut fixture, 9_241_002);

    let base_education = get_json(&fixture, &format!("/api/resume/{}/education", base_id));
    let education_id = base_education["body"][0]["id"].as_i64().unwrap();

    let deactivate_status = fixture
        .client()
        .put(format!("/api/education/{}", education_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "active": false }).to_string())
        .dispatch()
        .status();
    assert_eq!(deactivate_status, Status::Ok, "deactivate base education");

    let base_education = get_json(&fixture, &format!("/api/resume/{}/education", base_id));
    assert_eq!(base_education["body"][0]["active"], Value::Bool(false));

    let variant = create_variant(fixture.user_id(), base_id, clone_request()).expect("clone");
    fixture.track_resume_id(variant.id);

    let cloned_education = get_json(&fixture, &format!("/api/resume/{}/education", variant.id));
    let cloned_item = &cloned_education["body"][0];

    assert_eq!(
        cloned_item["active"],
        Value::Bool(false),
        "clone preserves the inactive flag"
    );
    assert_eq!(
        cloned_item["display_order"], base_education["body"][0]["display_order"],
        "clone preserves display order"
    );
}

#[test]
fn variant_create_nesting_returns_400() {
    let mut fixture = support::Fixture::new(9_241_003);
    let base_id = import_base(&mut fixture, 9_241_003);

    let variant = create_variant(fixture.user_id(), base_id, clone_request()).expect("clone");
    fixture.track_resume_id(variant.id);

    match create_variant(fixture.user_id(), variant.id, clone_request()) {
        Err(application::error::ApplicationError::BadRequest(message)) => {
            assert!(
                message.contains("variant"),
                "unexpected message: {}",
                message
            );
        }
        Err(other) => panic!("expected BadRequest, got {:?}", other),
        Ok(_) => panic!("expected BadRequest, but the clone succeeded"),
    }
}

#[test]
fn variant_create_non_owner_403_precedes_nesting_400() {
    let mut fixture = support::Fixture::new(9_241_004);
    let base_id = import_base(&mut fixture, 9_241_004);

    let variant = create_variant(fixture.user_id(), base_id, clone_request()).expect("clone");
    fixture.track_resume_id(variant.id);

    let other = support::register_and_login(fixture.client(), "variant.other");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    match create_variant(other.user_id, variant.id, clone_request()) {
        Err(application::error::ApplicationError::Forbidden) => {}
        Err(other) => panic!("expected Forbidden, got {:?}", other),
        Ok(_) => panic!("expected Forbidden, but the clone succeeded"),
    }
}

#[test]
fn variant_create_defaults_public_from_base_and_tag_true() {
    let mut fixture = support::Fixture::new(9_241_005);
    let base_id = import_base(&mut fixture, 9_241_005);

    let variant = create_variant(fixture.user_id(), base_id, clone_request()).expect("clone");
    fixture.track_resume_id(variant.id);

    assert!(variant.is_public, "base is public, so the clone is public");
    assert!(variant.show_variant_tag, "tag defaults to true");
    assert_eq!(variant.base_resume_id, Some(base_id));
    assert_eq!(variant.created_by, Some(fixture.user_id()));
    assert_eq!(variant.company_name.as_deref(), Some("Acme Corp"));
    assert_eq!(variant.variant_label.as_deref(), Some("Acme 2026"));

    let mut hidden = clone_request();
    hidden.is_public = Some(false);
    hidden.show_variant_tag = Some(false);
    let private = create_variant(fixture.user_id(), base_id, hidden).expect("clone private");
    fixture.track_resume_id(private.id);

    assert!(!private.is_public, "explicit is_public wins over the base");
    assert!(!private.show_variant_tag, "explicit tag value is stored");
}

fn post_variant(
    fixture: &support::Fixture,
    base_id: i32,
    body: Value,
) -> rocket::local::blocking::LocalResponse<'_> {
    fixture
        .client()
        .post(format!("/api/resume/{}/variants", base_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch()
}

fn post_variant_json(fixture: &support::Fixture, base_id: i32, body: Value) -> (Status, Value) {
    let response = post_variant(fixture, base_id, body);
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn get_variants_json(
    fixture: &support::Fixture,
    base_id: i32,
    token: Option<&str>,
) -> (Status, Value) {
    let request = fixture
        .client()
        .get(format!("/api/resume/{}/variants", base_id));
    let request = match token {
        Some(token) => request.header(support::auth_header(token)),
        None => request,
    };
    let response = request.dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn variant_body() -> Value {
    serde_json::json!({
        "company_name": "Acme Corp",
        "role_title": "Backend Engineer",
        "target_date": "2026-03",
        "job_description": "Build the thing",
        "variant_label": "Acme 2026"
    })
}

#[test]
fn variant_list_endpoint_visibility_and_empty_on_variant() {
    let mut fixture = support::Fixture::new(9_241_014);
    let base_id = import_base(&mut fixture, 9_241_014);

    let (public_status, public_json) = post_variant_json(&fixture, base_id, variant_body());
    assert_eq!(public_status, Status::Created);
    let public_id = public_json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(public_id);

    let mut hidden_body = variant_body();
    hidden_body["is_public"] = serde_json::json!(false);
    let (hidden_status, hidden_json) = post_variant_json(&fixture, base_id, hidden_body);
    assert_eq!(hidden_status, Status::Created);
    let hidden_id = hidden_json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(hidden_id);

    let owner_token = fixture.auth_token().to_string();
    let (owner_status, owner_json) = get_variants_json(&fixture, base_id, Some(&owner_token));
    assert_eq!(owner_status, Status::Ok);
    let owner_items = owner_json["body"].as_array().unwrap();
    assert_eq!(owner_items.len(), 2, "owner sees every variant");

    let (anonymous_status, anonymous_json) = get_variants_json(&fixture, base_id, None);
    assert_eq!(anonymous_status, Status::Ok);
    let anonymous_items = anonymous_json["body"].as_array().unwrap();
    assert_eq!(anonymous_items.len(), 1, "anonymous sees public variants");
    assert_eq!(anonymous_items[0]["id"].as_i64().unwrap() as i32, public_id);

    let (variant_status, variant_json) = get_variants_json(&fixture, public_id, Some(&owner_token));
    assert_eq!(variant_status, Status::Ok);
    assert!(
        variant_json["body"].as_array().unwrap().is_empty(),
        "listing a variant returns an empty list"
    );
}

#[test]
fn variant_list_endpoint_private_base_returns_404_for_other_user() {
    let mut fixture = support::Fixture::new(9_241_015);
    let base_id = import_base_with_visibility(&mut fixture, 9_241_015, false);

    let (status, json) = post_variant_json(&fixture, base_id, variant_body());
    assert_eq!(status, Status::Created);
    fixture.track_resume_id(json["body"]["id"].as_i64().unwrap() as i32);

    let other = support::register_and_login(fixture.client(), "variant.other.get");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    let (other_status, _) = get_variants_json(&fixture, base_id, Some(&other.token));
    assert_eq!(other_status, Status::NotFound, "private base stays hidden");

    let owner_token = fixture.auth_token().to_string();
    let (owner_status, owner_json) = get_variants_json(&fixture, base_id, Some(&owner_token));
    assert_eq!(owner_status, Status::Ok);
    assert_eq!(owner_json["body"].as_array().unwrap().len(), 1);
}

#[test]
fn variant_create_endpoint_returns_201_and_publishes_for_variant_id_only() {
    let mut fixture = support::Fixture::new(9_241_010);
    let base_id = import_base(&mut fixture, 9_241_010);

    let mut base_events = fixture.hub.subscribe(base_id);

    let response = post_variant(&fixture, base_id, variant_body());
    assert_eq!(response.status(), Status::Created, "create variant");

    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let variant_id = json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    assert_ne!(variant_id, base_id, "the clone is a new row");
    assert!(
        base_events.try_recv().is_err(),
        "subscribers of the base are not notified"
    );

    // The clone publishes under its own id, so a subscriber on the variant
    // receives later variant-scoped changes.
    let mut variant_events = fixture.hub.subscribe(variant_id);
    let update_status = fixture
        .client()
        .put(format!("/api/resume/{}", variant_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "variant_label": "Acme 2026 v2" }).to_string())
        .dispatch()
        .status();
    assert_eq!(update_status, Status::Ok);

    let event = variant_events.try_recv().expect("variant event");
    assert_eq!(event.resume_id, variant_id);
    assert!(
        base_events.try_recv().is_err(),
        "base subscribers stay silent"
    );
}

#[test]
fn variant_create_endpoint_rejects_nesting_with_400() {
    let mut fixture = support::Fixture::new(9_241_011);
    let base_id = import_base(&mut fixture, 9_241_011);

    let response = post_variant(&fixture, base_id, variant_body());
    assert_eq!(response.status(), Status::Created);
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let variant_id = json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let nested = post_variant(&fixture, variant_id, variant_body());
    assert_eq!(nested.status(), Status::BadRequest, "no nested variants");
}

#[test]
fn variant_create_endpoint_forbids_non_owner_with_403() {
    let mut fixture = support::Fixture::new(9_241_012);
    let base_id = import_base(&mut fixture, 9_241_012);

    let response = post_variant(&fixture, base_id, variant_body());
    assert_eq!(response.status(), Status::Created);
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let variant_id = json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let other = support::register_and_login(fixture.client(), "variant.other.post");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    // The base is public, so the other user can read it, but only the owner
    // may clone. Ownership is checked before the nesting rule, so cloning the
    // variant answers 403 rather than revealing that it is a variant.
    let forbidden = fixture
        .client()
        .post(format!("/api/resume/{}/variants", variant_id))
        .header(support::auth_header(&other.token))
        .header(ContentType::JSON)
        .body(variant_body().to_string())
        .dispatch();
    assert_eq!(forbidden.status(), Status::Forbidden);
}

#[test]
fn variant_create_endpoint_missing_base_returns_404() {
    let mut fixture = support::Fixture::new(9_241_013);

    let response = post_variant(&fixture, 2_000_000_000, variant_body());
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn variant_list_visibility_owner_vs_public() {
    let mut fixture = support::Fixture::new(9_241_006);
    let base_id = import_base(&mut fixture, 9_241_006);

    let public_variant =
        create_variant(fixture.user_id(), base_id, clone_request()).expect("clone public");
    fixture.track_resume_id(public_variant.id);

    let mut hidden_request = clone_request();
    hidden_request.is_public = Some(false);
    let private_variant =
        create_variant(fixture.user_id(), base_id, hidden_request).expect("clone private");
    fixture.track_resume_id(private_variant.id);

    let owner_view = list_variants(base_id, Some(fixture.user_id())).expect("owner listing");
    assert_eq!(owner_view.len(), 2, "owner sees every variant");

    let anonymous_view = list_variants(base_id, None).expect("anonymous listing");
    assert_eq!(
        anonymous_view.len(),
        1,
        "anonymous sees public variants only"
    );
    assert_eq!(anonymous_view[0].id, public_variant.id);

    let other = support::register_and_login(fixture.client(), "variant.other.list");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    let other_view = list_variants(base_id, Some(other.user_id)).expect("other listing");
    assert_eq!(
        other_view.len(),
        1,
        "another user sees public variants only"
    );
    assert_eq!(other_view[0].id, public_variant.id);
}

#[test]
fn variant_list_private_base_hidden_from_non_owner() {
    let mut fixture = support::Fixture::new(9_241_007);
    let base_id = import_base_with_visibility(&mut fixture, 9_241_007, false);

    let variant = create_variant(fixture.user_id(), base_id, clone_request()).expect("clone");
    fixture.track_resume_id(variant.id);

    let other = support::register_and_login(fixture.client(), "variant.other.private");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    match list_variants(base_id, Some(other.user_id)) {
        Err(ApplicationError::NotFound(_)) => {}
        Err(other_err) => panic!("expected NotFound, got {:?}", other_err),
        Ok(_) => panic!("expected NotFound, but the listing succeeded"),
    }

    assert_eq!(
        list_variants(base_id, Some(fixture.user_id()))
            .expect("owner listing")
            .len(),
        1
    );
}
