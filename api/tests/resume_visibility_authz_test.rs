use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

#[test]
fn test_public_private_visibility_and_owner_only_enforcement() {
    let mut fixture = support::Fixture::new(9_225_340);

    let user_a = support::register_and_login(fixture.client(), "visibility.owner");
    fixture.track_user_id(user_a.user_id);
    fixture.track_session_id(user_a.token.clone());

    let user_b = support::register_and_login(fixture.client(), "visibility.other");
    fixture.track_user_id(user_b.user_id);
    fixture.track_session_id(user_b.token.clone());

    let public_email = support::unique_email("public.resume");
    let public_resume_payload = serde_json::json!({
        "name": "Public Resume",
        "profile_image_url": null,
        "location": "Somewhere",
        "email": public_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true
    });

    let public_create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(&user_a.token))
        .header(ContentType::JSON)
        .body(public_resume_payload.to_string())
        .dispatch();

    assert_eq!(public_create_response.status(), Status::Created);

    let public_create_body = public_create_response.into_string().expect("public create body");
    let public_create_json: Value = serde_json::from_str(&public_create_body).expect("valid json");
    let public_resume_id = public_create_json["body"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(public_resume_id);

    let private_email = support::unique_email("private.resume");
    let private_resume_payload = serde_json::json!({
        "name": "Private Resume",
        "profile_image_url": null,
        "location": "Hidden",
        "email": private_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let private_create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(&user_a.token))
        .header(ContentType::JSON)
        .body(private_resume_payload.to_string())
        .dispatch();

    assert_eq!(private_create_response.status(), Status::Created);

    let private_create_body = private_create_response.into_string().expect("private create body");
    let private_create_json: Value =
        serde_json::from_str(&private_create_body).expect("valid json");
    let private_resume_id = private_create_json["body"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(private_resume_id);

    let anon_list_response = fixture.client().get("/api/resumes").dispatch();
    assert_eq!(anon_list_response.status(), Status::Ok);
    let anon_list_body = anon_list_response.into_string().expect("anon list body");
    let anon_list_json: Value = serde_json::from_str(&anon_list_body).expect("valid json");
    let anon_items = anon_list_json["body"].as_array().expect("array");

    assert!(anon_items
        .iter()
        .any(|r| r["id"].as_i64() == Some(public_resume_id as i64)));
    assert!(!anon_items
        .iter()
        .any(|r| r["id"].as_i64() == Some(private_resume_id as i64)));

    let owner_list_response = fixture
        .client()
        .get("/api/resumes")
        .header(support::auth_header(&user_a.token))
        .dispatch();
    assert_eq!(owner_list_response.status(), Status::Ok);
    let owner_list_body = owner_list_response.into_string().expect("owner list body");
    let owner_list_json: Value = serde_json::from_str(&owner_list_body).expect("valid json");
    let owner_items = owner_list_json["body"].as_array().expect("array");

    assert!(owner_items
        .iter()
        .any(|r| r["id"].as_i64() == Some(public_resume_id as i64)));
    assert!(owner_items
        .iter()
        .any(|r| r["id"].as_i64() == Some(private_resume_id as i64)));

    let anon_get_private = fixture
        .client()
        .get(format!("/api/resume/{}", private_resume_id))
        .dispatch();
    assert_eq!(anon_get_private.status(), Status::NotFound);

    let other_get_private = fixture
        .client()
        .get(format!("/api/resume/{}", private_resume_id))
        .header(support::auth_header(&user_b.token))
        .dispatch();
    assert_eq!(other_get_private.status(), Status::NotFound);

    let owner_get_private = fixture
        .client()
        .get(format!("/api/resume/{}", private_resume_id))
        .header(support::auth_header(&user_a.token))
        .dispatch();
    assert_eq!(owner_get_private.status(), Status::Ok);

    let other_update_private_payload = serde_json::json!({
        "location": "Hacked"
    });

    let other_update_private = fixture
        .client()
        .put(format!("/api/resume/{}", private_resume_id))
        .header(support::auth_header(&user_b.token))
        .header(ContentType::JSON)
        .body(other_update_private_payload.to_string())
        .dispatch();
    assert_eq!(other_update_private.status(), Status::Forbidden);

    let other_delete_private = fixture
        .client()
        .delete(format!("/api/resume/{}", private_resume_id))
        .header(support::auth_header(&user_b.token))
        .dispatch();
    assert_eq!(other_delete_private.status(), Status::Forbidden);

    let other_create_skill_payload = serde_json::json!({
        "skill_name": "Rust",
        "confidence_percentage": 80,
        "display_order": 0
    });

    let other_create_skill = fixture
        .client()
        .post(format!("/api/resume/{}/skills", private_resume_id))
        .header(support::auth_header(&user_b.token))
        .header(ContentType::JSON)
        .body(other_create_skill_payload.to_string())
        .dispatch();
    assert_eq!(other_create_skill.status(), Status::Forbidden);
}
