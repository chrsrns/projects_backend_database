use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

#[test]
fn test_create_and_read_video() {
    let mut fixture = support::Fixture::new(9_229_001);

    let email = support::unique_email("video.create");
    let video_url = "https://example.com/video.mp4";
    let new_resume_json = serde_json::json!({
        "name": "Video User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "video": video_url
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

    assert_eq!(created_resume["video"], video_url);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];

    assert_eq!(retrieved_resume["id"], resume_id);
    assert_eq!(retrieved_resume["video"], video_url);
}

#[test]
fn test_update_video() {
    let mut fixture = support::Fixture::new(9_229_002);

    let email = support::unique_email("video.update");
    let new_resume_json = serde_json::json!({
        "name": "Update Video User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "video": "https://example.com/old.mp4"
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

    let updated_video = "https://example.com/new.mp4";
    let update_json = serde_json::json!({
        "video": updated_video
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
    assert_eq!(updated_resume["video"], updated_video);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];
    assert_eq!(retrieved_resume["video"], updated_video);
}

#[test]
fn test_update_video_with_null_clears() {
    let mut fixture = support::Fixture::new(9_229_003);

    let email = support::unique_email("video.null");
    let new_resume_json = serde_json::json!({
        "name": "Null Video User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "video": "https://example.com/video.mp4"
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
        "video": null
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
    assert!(retrieved_resume["video"].is_null());
}

#[test]
fn test_update_video_with_empty_string_no_change() {
    let mut fixture = support::Fixture::new(9_229_004);

    let email = support::unique_email("video.empty");
    let video_url = "https://example.com/video.mp4";
    let new_resume_json = serde_json::json!({
        "name": "Empty Video User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "video": video_url
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
        "video": ""
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
    assert_eq!(retrieved_resume["video"], video_url);
}

#[test]
fn test_create_video_too_long() {
    let fixture = support::Fixture::new(9_229_005);

    let email = support::unique_email("video.long");
    let long_video = "x".repeat(501);
    let new_resume_json = serde_json::json!({
        "name": "Long Video User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "video": long_video
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
fn test_create_video_blank_normalized() {
    let mut fixture = support::Fixture::new(9_229_006);

    let email = support::unique_email("video.blank");
    let new_resume_json = serde_json::json!({
        "name": "Blank Video User",
        "profile_image_url": null,
        "location": null,
        "email": email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true,
        "video": "   \t\n  "
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

    assert!(created_resume["video"].is_null());

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("get body");
    let get_json: Value = serde_json::from_str(&get_body).expect("valid JSON");
    let retrieved_resume = &get_json["body"];
    assert!(retrieved_resume["video"].is_null());
}
