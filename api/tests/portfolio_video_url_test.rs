use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn create_resume(fixture: &mut support::Fixture) -> i32 {
    let email = support::unique_email("portfolio_video.resume");
    let new_resume_json = serde_json::json!({
        "name": "Portfolio Video User",
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

fn create_project(
    fixture: &mut support::Fixture,
    resume_id: i32,
    video_url: Option<&str>,
) -> (i32, Value) {
    let mut new_project_json = serde_json::json!({
        "project_name": "Resume Builder",
        "image_url": null,
        "project_link": null,
        "source_code_link": null,
        "description": "A project",
        "display_order": 0
    });

    if let Some(url) = video_url {
        new_project_json["video_url"] = Value::String(url.to_string());
    } else {
        new_project_json["video_url"] = Value::Null;
    }

    let create_response = fixture
        .client()
        .post(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_project_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
    let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;
    (project_id, create_json["body"].clone())
}

#[test]
fn test_create_and_read_portfolio_video_url() {
    let mut fixture = support::Fixture::new(9_230_001);
    let resume_id = create_resume(&mut fixture);
    let video_url = "https://example.com/video.mp4";
    let (project_id, create_body) = create_project(&mut fixture, resume_id, Some(video_url));

    assert_eq!(create_body["video_url"], video_url);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let projects = get_json["body"].as_array().expect("projects array");
    let project = projects
        .iter()
        .find(|p| p["id"].as_i64() == Some(project_id as i64))
        .expect("project in list");
    assert_eq!(project["video_url"], video_url);
}

#[test]
fn test_update_portfolio_video_url() {
    let mut fixture = support::Fixture::new(9_230_002);
    let resume_id = create_resume(&mut fixture);
    let (project_id, _) = create_project(
        &mut fixture,
        resume_id,
        Some("https://example.com/old.mp4"),
    );

    let updated_video = "https://example.com/new.mp4";
    let update_json = serde_json::json!({
        "video_url": updated_video
    });

    let update_response = fixture
        .client()
        .put(format!("/api/portfolio_projects/{}", project_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_json.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let update_body = update_response.into_string().expect("update body");
    let update_json: Value = serde_json::from_str(&update_body).expect("valid JSON");
    let updated_project = &update_json["body"];
    assert_eq!(updated_project["id"], project_id);
    assert_eq!(updated_project["video_url"], updated_video);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let projects = get_json["body"].as_array().expect("projects array");
    let project = projects
        .iter()
        .find(|p| p["id"].as_i64() == Some(project_id as i64))
        .expect("project in list");
    assert_eq!(project["video_url"], updated_video);
}

#[test]
fn test_update_portfolio_video_url_with_null_clears() {
    let mut fixture = support::Fixture::new(9_230_003);
    let resume_id = create_resume(&mut fixture);
    let (project_id, _) = create_project(
        &mut fixture,
        resume_id,
        Some("https://example.com/video.mp4"),
    );

    let update_json = serde_json::json!({
        "video_url": null
    });

    let update_response = fixture
        .client()
        .put(format!("/api/portfolio_projects/{}", project_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_json.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let projects = get_json["body"].as_array().expect("projects array");
    let project = projects
        .iter()
        .find(|p| p["id"].as_i64() == Some(project_id as i64))
        .expect("project in list");
    assert!(project["video_url"].is_null());
}

#[test]
fn test_update_portfolio_video_url_with_empty_string_no_change() {
    let mut fixture = support::Fixture::new(9_230_004);
    let resume_id = create_resume(&mut fixture);
    let video_url = "https://example.com/video.mp4";
    let (project_id, _) = create_project(&mut fixture, resume_id, Some(video_url));

    let update_json = serde_json::json!({
        "video_url": ""
    });

    let update_response = fixture
        .client()
        .put(format!("/api/portfolio_projects/{}", project_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_json.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let projects = get_json["body"].as_array().expect("projects array");
    let project = projects
        .iter()
        .find(|p| p["id"].as_i64() == Some(project_id as i64))
        .expect("project in list");
    assert_eq!(project["video_url"], video_url);
}

#[test]
fn test_create_portfolio_video_url_too_long() {
    let mut fixture = support::Fixture::new(9_230_005);
    let resume_id = create_resume(&mut fixture);
    let long_url = "x".repeat(501);

    let new_project_json = serde_json::json!({
        "project_name": "Resume Builder",
        "image_url": null,
        "project_link": null,
        "source_code_link": null,
        "video_url": long_url,
        "description": "A project",
        "display_order": 0
    });

    let create_response = fixture
        .client()
        .post(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_project_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::BadRequest);
}

#[test]
fn test_create_portfolio_video_url_blank_normalized() {
    let mut fixture = support::Fixture::new(9_230_006);
    let resume_id = create_resume(&mut fixture);

    let new_project_json = serde_json::json!({
        "project_name": "Resume Builder",
        "image_url": null,
        "project_link": null,
        "source_code_link": null,
        "video_url": "   \t\n  ",
        "description": "A project",
        "display_order": 0
    });

    let create_response = fixture
        .client()
        .post(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_project_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid JSON");
    let project_id = create_json["body"]["id"].as_i64().expect("Project ID") as i32;

    assert!(create_json["body"]["video_url"].is_null());

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let projects = get_json["body"].as_array().expect("projects array");
    let project = projects
        .iter()
        .find(|p| p["id"].as_i64() == Some(project_id as i64))
        .expect("project in list");
    assert!(project["video_url"].is_null());
}
