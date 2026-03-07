use rocket::http::{ContentType, Status};
use serde_json::Value;

use api::realtime::ResumeChangedAction;

mod support;

#[test]
fn test_resume_update_publishes_resume_changed_event() {
    let mut fixture = support::Fixture::new(9_225_339);

    let unique_email = format!(
        "realtime.resume.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Realtime User",
        "profile_image_url": null,
        "location": "Initial",
        "email": unique_email,
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
    let create_json: Value = serde_json::from_str(&create_body).expect("valid json");
    let resume_id = create_json["body"]["id"].as_i64().expect("id") as i32;

    fixture.track_resume_id(resume_id);

    let mut rx = fixture.hub.subscribe(resume_id);

    let update_payload = serde_json::json!({
        "location": "Updated"
    });

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_payload.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let evt = rx.try_recv().expect("resume.changed event");
    assert_eq!(evt.resume_id, resume_id);
    assert_eq!(
        evt.action,
        ResumeChangedAction::Updated(api::realtime::SectionType::PersonalInfo)
    );
}

#[test]
fn test_skill_create_publishes_resume_changed_event() {
    let mut fixture = support::Fixture::new(9_225_340);

    let unique_email = format!(
        "realtime.skills.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Realtime Skills User",
        "profile_image_url": null,
        "location": "Initial",
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
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
    let create_json: Value = serde_json::from_str(&create_body).expect("valid json");
    let resume_id = create_json["body"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(resume_id);

    let mut rx = fixture.hub.subscribe(resume_id);

    let new_skill_json = serde_json::json!({
        "skill_name": "Rust",
        "confidence_percentage": 80,
        "display_order": 0
    });

    let create_skill_response = fixture
        .client()
        .post(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_skill_json.to_string())
        .dispatch();

    assert_eq!(create_skill_response.status(), Status::Created);

    let evt = rx.try_recv().expect("resume.changed event");
    assert_eq!(evt.resume_id, resume_id);
    assert_eq!(evt.action, ResumeChangedAction::Updated);
}
