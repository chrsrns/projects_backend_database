use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn create_resume(fixture: &mut support::Fixture) -> i32 {
    let email = support::unique_email("portfolio_url_fields.resume");
    let new_resume_json = serde_json::json!({
        "name": "Portfolio URL Fields User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
    let resume_id = create_json["body"]["id"].as_i64().expect("Resume ID") as i32;
    fixture.track_resume_id(resume_id);
    resume_id
}

fn post_create_project<'a>(
    fixture: &'a mut support::Fixture,
    resume_id: i32,
    field: &'a str,
    value: Option<&'a str>,
) -> rocket::local::blocking::LocalResponse<'a> {
    let mut new_project_json = serde_json::json!({
        "project_name": "URL Field Project",
        "image_url": null,
        "project_link": null,
        "source_code_link": null,
        "video_url": null,
        "description": "A project",
        "display_order": 0
    });

    if let Some(v) = value {
        new_project_json[field] = Value::String(v.to_string());
    } else {
        new_project_json[field] = Value::Null;
    }

    fixture
        .client()
        .post(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_project_json.to_string())
        .dispatch()
}

fn post_update_project<'a>(
    fixture: &'a mut support::Fixture,
    project_id: i32,
    field: &'a str,
    value: Option<&'a str>,
) -> rocket::local::blocking::LocalResponse<'a> {
    let mut update_json = serde_json::json!({});

    if let Some(v) = value {
        update_json[field] = Value::String(v.to_string());
    } else {
        update_json[field] = Value::Null;
    }

    fixture
        .client()
        .put(format!("/api/portfolio_projects/{}", project_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_json.to_string())
        .dispatch()
}

fn get_project(
    fixture: &mut support::Fixture,
    resume_id: i32,
    project_id: i32,
) -> Value {
    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let projects = get_json["body"].as_array().expect("projects array");
    projects
        .iter()
        .find(|p| p["id"].as_i64() == Some(project_id as i64))
        .expect("project in list")
        .clone()
}

const URL_FIELDS: &[&str] = &["image_url", "project_link", "source_code_link"];

#[test]
fn test_url_fields_create_and_read() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_001 + i as i64);
        let resume_id = create_resume(&mut fixture);
        let url = "https://example.com/value";

        let create_response = post_create_project(&mut fixture, resume_id, field, Some(url));
        assert_eq!(create_response.status(), Status::Created, "create {}: 201", field);

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        assert_eq!(create_json["body"][field], url, "create response {} preserved", field);

        let project = get_project(&mut fixture, resume_id, project_id);
        assert_eq!(project[field], url, "read {} preserved", field);
    }
}

#[test]
fn test_url_fields_create_blank_normalized() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_010 + i as i64);
        let resume_id = create_resume(&mut fixture);

        let create_response =
            post_create_project(&mut fixture, resume_id, field, Some("   \t\n  "));
        assert_eq!(
            create_response.status(),
            Status::Created,
            "create {} blank: 201",
            field
        );

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        assert!(create_json["body"][field].is_null(), "create response {} blank null", field);

        let project = get_project(&mut fixture, resume_id, project_id);
        assert!(project[field].is_null(), "read {} blank null", field);
    }
}

#[test]
fn test_url_fields_create_500_limit() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_020 + i as i64);
        let resume_id = create_resume(&mut fixture);
        let long = "x".repeat(501);

        let create_response = post_create_project(&mut fixture, resume_id, field, Some(&long));
        assert_eq!(
            create_response.status(),
            Status::BadRequest,
            "create {} >500: 400",
            field
        );
    }
}

#[test]
fn test_url_fields_create_500_exact() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_030 + i as i64);
        let resume_id = create_resume(&mut fixture);
        let url = "x".repeat(500);

        let create_response = post_create_project(&mut fixture, resume_id, field, Some(&url));
        assert_eq!(
            create_response.status(),
            Status::Created,
            "create {} 500 exact: 201",
            field
        );

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        assert_eq!(
            create_json["body"][field].as_str().expect("string"),
            url,
            "create response {} 500 exact",
            field
        );

        let project = get_project(&mut fixture, resume_id, project_id);
        assert_eq!(project[field].as_str().expect("string"), url, "read {} 500 exact", field);
    }
}

#[test]
fn test_url_fields_update() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_040 + i as i64);
        let resume_id = create_resume(&mut fixture);
        let old_url = "https://example.com/old";
        let new_url = "https://example.com/new";

        let create_response = post_create_project(&mut fixture, resume_id, field, Some(old_url));
        assert_eq!(create_response.status(), Status::Created);

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        let update_response = post_update_project(&mut fixture, project_id, field, Some(new_url));
        assert_eq!(
            update_response.status(),
            Status::Ok,
            "update {}: 200",
            field
        );
        drop(update_response);

        let project = get_project(&mut fixture, resume_id, project_id);
        assert_eq!(project[field], new_url, "read {} updated", field);
    }
}

#[test]
fn test_url_fields_update_null_clears() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_050 + i as i64);
        let resume_id = create_resume(&mut fixture);

        let create_response =
            post_create_project(&mut fixture, resume_id, field, Some("https://example.com/old"));
        assert_eq!(create_response.status(), Status::Created);

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        let update_response = post_update_project(&mut fixture, project_id, field, None);
        assert_eq!(
            update_response.status(),
            Status::Ok,
            "update {} null: 200",
            field
        );
        drop(update_response);

        let project = get_project(&mut fixture, resume_id, project_id);
        assert!(project[field].is_null(), "read {} null cleared", field);
    }
}

#[test]
fn test_url_fields_update_empty_string_no_change() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_060 + i as i64);
        let resume_id = create_resume(&mut fixture);
        let url = "https://example.com/old";

        let create_response = post_create_project(&mut fixture, resume_id, field, Some(url));
        assert_eq!(create_response.status(), Status::Created);

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        let update_response = post_update_project(&mut fixture, project_id, field, Some(""));
        assert_eq!(
            update_response.status(),
            Status::Ok,
            "update {} empty: 200",
            field
        );
        drop(update_response);

        let project = get_project(&mut fixture, resume_id, project_id);
        assert_eq!(project[field], url, "read {} empty no change", field);
    }
}

#[test]
fn test_url_fields_update_blank_normalized() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_070 + i as i64);
        let resume_id = create_resume(&mut fixture);

        let create_response =
            post_create_project(&mut fixture, resume_id, field, Some("https://example.com/old"));
        assert_eq!(create_response.status(), Status::Created);

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        let update_response =
            post_update_project(&mut fixture, project_id, field, Some("   \t\n  "));
        assert_eq!(
            update_response.status(),
            Status::Ok,
            "update {} blank: 200",
            field
        );
        drop(update_response);

        let project = get_project(&mut fixture, resume_id, project_id);
        assert!(project[field].is_null(), "read {} blank null", field);
    }
}

#[test]
fn test_url_fields_update_500_limit() {
    for (i, field) in URL_FIELDS.iter().enumerate() {
        let mut fixture = support::Fixture::new(9_240_080 + i as i64);
        let resume_id = create_resume(&mut fixture);

        let create_response =
            post_create_project(&mut fixture, resume_id, field, Some("https://example.com/old"));
        assert_eq!(create_response.status(), Status::Created);

        let create_body = create_response.into_string().expect("create body");
        let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
        let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

        let long = "x".repeat(501);
        let update_response = post_update_project(&mut fixture, project_id, field, Some(&long));
        assert_eq!(
            update_response.status(),
            Status::BadRequest,
            "update {} >500: 400",
            field
        );
    }
}
