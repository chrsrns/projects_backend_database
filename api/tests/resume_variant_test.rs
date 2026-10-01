use application::resume::variant::create_variant;
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

fn variant_markdown(email: &str) -> String {
    format!(
        "# Jane Doe\n\n- Location: New York, NY\n- Email: {}\n- GitHub: https://github.com/janedoe\n- Mobile: +1987654321\n- Public: true\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024)\n- Degree: Bachelor of Science\n- Description: Focused on software engineering\n- Graduated with honors\n- Specialized in distributed systems\n\n## Skills\n\n- Rust - 90%\n- Python - 75%\n- JavaScript - 60%\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (Jan 2020 - Present)\n- Description: Backend development\n- Led team of 5 developers\n- Improved system performance by 40%\n\n## Portfolio Projects\n\n### My Portfolio\n- Live: https://example.com\n- Source: https://github.com/janedoe/project\n- Technologies: Rust, Rocket, Diesel\n- Built a scalable resume API\n\n## Languages & Frameworks\n\n### Rust\n- Rocket\n- Actix\n\n### Python\n- Django\n- FastAPI\n",
        email
    )
}

fn import_base(fixture: &mut support::Fixture, lock_key: i64) -> i32 {
    let email = format!("variant.base.{}.{}@example.com", lock_key, unique_suffix());
    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(variant_markdown(&email))
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
