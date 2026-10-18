use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn create_resume(fixture: &mut support::Fixture, visibility: &str) -> uuid::Uuid {
    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Matrix Resume",
                "email": "matrix.owner@example.com",
                "visibility": visibility,
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let id = support::resume_uuid(&json["body"]["id"]);
    fixture.track_resume_id(id);
    id
}

fn create_child(fixture: &support::Fixture, path: &str, body: Value) -> i64 {
    let response = fixture
        .client()
        .post(path.to_string())
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::Created, "create {path}");
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    json["body"]["id"].as_i64().expect("child id")
}

/// Every mutating route reachable on a public resume must answer 403 to a
/// non-owner: child creates, child and grandchild update/delete, markdown
/// import, resume update/delete. Existing suites already cover variant
/// create, revision restore/history, and the private-visibility 403s.
#[test]
fn non_owner_mutations_forbidden_across_all_sections() {
    let mut fixture = support::Fixture::new(9_260_001);
    let resume_id = create_resume(&mut fixture, "public");

    let skill = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/skills"),
        serde_json::json!({"skill_name": "Rust", "confidence_percentage": 80, "display_order": 0}),
    );
    let language = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/languages"),
        serde_json::json!({"language_name": "TypeScript", "display_order": 0}),
    );
    let education = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/education"),
        serde_json::json!({"education_stage": "University", "institution_name": "U", "start_date": "2020-01-01", "end_date": "2024-01-01", "description": "d", "display_order": 0}),
    );
    let work = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/work_experiences"),
        serde_json::json!({"job_title": "Engineer", "start_date": "2022-01-01", "end_date": null, "description": "d", "display_order": 0}),
    );
    let project = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/portfolio_projects"),
        serde_json::json!({"project_name": "Builder", "image_url": null, "project_link": null, "source_code_link": null, "description": "d", "display_order": 0}),
    );

    let edu_kp = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/education/{education}/key_points"),
        serde_json::json!({"key_point": "Graduated"}),
    );
    let work_kp = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/work_experiences/{work}/key_points"),
        serde_json::json!({"key_point": "Led a project"}),
    );
    let portfolio_kp = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/portfolio_projects/{project}/key_points"),
        serde_json::json!({"key_point": "Built it"}),
    );
    let tech = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/portfolio_projects/{project}/technologies"),
        serde_json::json!({"technology_name": "Rust", "display_order": 0}),
    );
    let framework = create_child(
        &fixture,
        &format!("/api/resume/{resume_id}/languages/{language}/frameworks"),
        serde_json::json!({"framework_name": "Rocket", "display_order": 0}),
    );

    let non_owner = support::register_and_login(fixture.client(), "matrix.nonowner");
    fixture.track_user_id(non_owner.user_id);
    fixture.track_session_id(non_owner.token.clone());
    let header = support::auth_header(&non_owner.token);

    // Bodies must deserialize: the JSON guard runs before the handler's
    // owner check, so each create carries a real minimal payload.

    let skill_body =
        serde_json::json!({"skill_name": "Rust", "confidence_percentage": 80}).to_string();
    let language_body = serde_json::json!({"language_name": "Rust"}).to_string();
    let education_body =
        serde_json::json!({"education_stage": "U", "institution_name": "U", "start_date": "2020-01-01", "display_order": 0}).to_string();
    let work_body =
        serde_json::json!({"job_title": "E", "start_date": "2022-01-01", "display_order": 0})
            .to_string();
    let project_body = serde_json::json!({"project_name": "P", "display_order": 0}).to_string();
    let kp_body = serde_json::json!({"key_point": "k"}).to_string();
    let tech_body = serde_json::json!({"technology_name": "Rust", "display_order": 0}).to_string();
    let framework_body =
        serde_json::json!({"framework_name": "Rocket", "display_order": 0}).to_string();
    let update_body = serde_json::json!({}).to_string();
    let markdown_body = "# Resume\n\n## Identity\n- Name: X\n- Email: x@y.z\n".to_string();

    let mutations: Vec<(&str, String, &str)> = vec![
        ("PUT", format!("/api/resume/{resume_id}"), &update_body),
        ("DELETE", format!("/api/resume/{resume_id}"), ""),
        (
            "POST",
            format!("/api/resume/{resume_id}/import/markdown"),
            &markdown_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/skills"),
            &skill_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/languages"),
            &language_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/education"),
            &education_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/work_experiences"),
            &work_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/portfolio_projects"),
            &project_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/education/{education}/key_points"),
            &kp_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/work_experiences/{work}/key_points"),
            &kp_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/portfolio_projects/{project}/key_points"),
            &kp_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/portfolio_projects/{project}/technologies"),
            &tech_body,
        ),
        (
            "POST",
            format!("/api/resume/{resume_id}/languages/{language}/frameworks"),
            &framework_body,
        ),
        ("PUT", format!("/api/skills/{skill}"), &update_body),
        ("DELETE", format!("/api/skills/{skill}"), ""),
        ("PUT", format!("/api/languages/{language}"), &update_body),
        ("DELETE", format!("/api/languages/{language}"), ""),
        ("PUT", format!("/api/education/{education}"), &update_body),
        ("DELETE", format!("/api/education/{education}"), ""),
        ("PUT", format!("/api/work_experiences/{work}"), &update_body),
        ("DELETE", format!("/api/work_experiences/{work}"), ""),
        (
            "PUT",
            format!("/api/portfolio_projects/{project}"),
            &update_body,
        ),
        ("DELETE", format!("/api/portfolio_projects/{project}"), ""),
        (
            "PUT",
            format!("/api/education_key_points/{edu_kp}"),
            &update_body,
        ),
        ("DELETE", format!("/api/education_key_points/{edu_kp}"), ""),
        (
            "PUT",
            format!("/api/work_experience_key_points/{work_kp}"),
            &update_body,
        ),
        (
            "DELETE",
            format!("/api/work_experience_key_points/{work_kp}"),
            "",
        ),
        (
            "PUT",
            format!("/api/portfolio_key_points/{portfolio_kp}"),
            &update_body,
        ),
        (
            "DELETE",
            format!("/api/portfolio_key_points/{portfolio_kp}"),
            "",
        ),
        (
            "PUT",
            format!("/api/portfolio_technologies/{tech}"),
            &update_body,
        ),
        ("DELETE", format!("/api/portfolio_technologies/{tech}"), ""),
        ("PUT", format!("/api/frameworks/{framework}"), &update_body),
        ("DELETE", format!("/api/frameworks/{framework}"), ""),
    ];

    for (method, path, body) in mutations {
        let request = match method {
            "PUT" => fixture.client().put(path.clone()),
            "DELETE" => fixture.client().delete(path.clone()),
            _ => fixture.client().post(path.clone()),
        };
        let content_type = if path.ends_with("import/markdown") {
            ContentType::new("text", "markdown")
        } else {
            ContentType::JSON
        };
        let response = request
            .header(header.clone())
            .header(content_type)
            .body(body.to_string())
            .dispatch();
        assert_eq!(
            response.status(),
            Status::Forbidden,
            "{method} {path} must answer 403 for a non-owner"
        );
    }
}
