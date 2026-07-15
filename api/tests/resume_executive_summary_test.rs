use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

#[test]
fn test_create_and_read_executive_summary() {
    let mut fixture = support::Fixture::new(9_228_002);

    let email = support::unique_email("exec.summary.create");
    let new_resume_json = serde_json::json!({
        "name": "Executive Summary User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "executive_summary": "Experienced engineer with a focus on systems design."
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
    let created_resume = &create_json["body"];
    let resume_id = created_resume["id"].as_i64().expect("Resume ID") as i32;
    fixture.track_resume_id(resume_id);

    assert_eq!(
        created_resume["executive_summary"],
        "Experienced engineer with a focus on systems design."
    );

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];

    assert_eq!(retrieved_resume["id"], resume_id);
    assert_eq!(
        retrieved_resume["executive_summary"],
        "Experienced engineer with a focus on systems design."
    );
}

#[test]
fn test_update_executive_summary() {
    let mut fixture = support::Fixture::new(9_228_003);

    let email = support::unique_email("exec.summary.update");
    let new_resume_json = serde_json::json!({
        "name": "Update Executive Summary User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "executive_summary": "Initial summary."
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
    let created_resume = &create_json["body"];
    let resume_id = created_resume["id"].as_i64().expect("Resume ID") as i32;
    fixture.track_resume_id(resume_id);

    let update_json = serde_json::json!({
        "executive_summary": "Updated summary text."
    });

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_json.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let update_body = update_response.into_string().expect("update body");
    let update_json: Value = serde_json::from_str(&update_body).expect("valid JSON");
    let updated_resume = &update_json["body"];
    assert_eq!(updated_resume["id"], resume_id);
    assert_eq!(updated_resume["executive_summary"], "Updated summary text.");

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];
    assert_eq!(
        retrieved_resume["executive_summary"],
        "Updated summary text."
    );
}

#[test]
fn test_update_executive_summary_with_null_clears() {
    let mut fixture = support::Fixture::new(9_228_004);

    let email = support::unique_email("exec.summary.null");
    let new_resume_json = serde_json::json!({
        "name": "Null Executive Summary User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "executive_summary": "Will be cleared."
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
    let created_resume = &create_json["body"];
    let resume_id = created_resume["id"].as_i64().expect("Resume ID") as i32;
    fixture.track_resume_id(resume_id);

    let update_json = serde_json::json!({
        "executive_summary": null
    });

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_json.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];
    assert!(retrieved_resume["executive_summary"].is_null());
}

#[test]
fn test_create_executive_summary_too_long() {
    let fixture = support::Fixture::new(9_228_005);

    let email = support::unique_email("exec.summary.long");
    let long_summary = "x".repeat(5001);
    let new_resume_json = serde_json::json!({
        "name": "Long Executive Summary User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "executive_summary": long_summary
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::BadRequest);
}

#[test]
fn test_create_executive_summary_blank_normalized() {
    let mut fixture = support::Fixture::new(9_228_006);

    let email = support::unique_email("exec.summary.blank");
    let new_resume_json = serde_json::json!({
        "name": "Blank Executive Summary User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "executive_summary": "   \t\n  "
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
    let created_resume = &create_json["body"];
    let resume_id = created_resume["id"].as_i64().expect("Resume ID") as i32;
    fixture.track_resume_id(resume_id);

    assert!(created_resume["executive_summary"].is_null());

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];
    assert!(retrieved_resume["executive_summary"].is_null());
}
