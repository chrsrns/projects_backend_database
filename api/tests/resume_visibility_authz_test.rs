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

    let public_create_body = public_create_response
        .into_string()
        .expect("public create body");
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

    let private_create_body = private_create_response
        .into_string()
        .expect("private create body");
    let private_create_json: Value =
        serde_json::from_str(&private_create_body).expect("valid json");
    let private_resume_id = private_create_json["body"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(private_resume_id);

    let anon_list_response = fixture.client().get("/api/resumes").dispatch();
    assert_eq!(anon_list_response.status(), Status::Ok);
    let anon_list_body = anon_list_response.into_string().expect("anon list body");
    let anon_list_json: Value = serde_json::from_str(&anon_list_body).expect("valid json");
    let anon_items = anon_list_json["body"].as_array().expect("array");

    assert!(
        anon_items
            .iter()
            .any(|r| r["id"].as_i64() == Some(public_resume_id as i64))
    );
    assert!(
        !anon_items
            .iter()
            .any(|r| r["id"].as_i64() == Some(private_resume_id as i64))
    );

    let owner_list_response = fixture
        .client()
        .get("/api/resumes")
        .header(support::auth_header(&user_a.token))
        .dispatch();
    assert_eq!(owner_list_response.status(), Status::Ok);
    let owner_list_body = owner_list_response.into_string().expect("owner list body");
    let owner_list_json: Value = serde_json::from_str(&owner_list_body).expect("valid json");
    let owner_items = owner_list_json["body"].as_array().expect("array");

    assert!(
        owner_items
            .iter()
            .any(|r| r["id"].as_i64() == Some(public_resume_id as i64))
    );
    assert!(
        owner_items
            .iter()
            .any(|r| r["id"].as_i64() == Some(private_resume_id as i64))
    );

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

#[test]
fn test_child_active_filter_for_non_owners() {
    let mut fixture = support::Fixture::new(9_225_341);

    let owner = support::register_and_login(fixture.client(), "child.owner");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let non_owner = support::register_and_login(fixture.client(), "child.nonowner");
    fixture.track_user_id(non_owner.user_id);
    fixture.track_session_id(non_owner.token.clone());

    // Create a public resume
    let resume_payload = serde_json::json!({
        "name": "Child Visibility Test",
        "profile_image_url": null,
        "location": "Test City",
        "email": support::unique_email("child.visibility"),
        "github_url": null,
        "mobile_number": null,
        "is_public": true
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(resume_payload.to_string())
        .dispatch();

    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("create body");
    let create_json: Value = serde_json::from_str(&create_body).expect("valid json");
    let resume_id = create_json["body"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(resume_id);

    // Create education with active and inactive key points
    let education_payload = serde_json::json!({
        "education_stage": "Bachelor's",
        "institution_name": "Test University",
        "degree": "Computer Science",
        "start_date": "2020-09-01",
        "end_date": "2024-05-01",
        "description": "Test education",
        "display_order": 1
    });

    let edu_response = fixture
        .client()
        .post(format!("/api/resume/{}/education", resume_id))
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(education_payload.to_string())
        .dispatch();

    assert_eq!(edu_response.status(), Status::Created);

    let edu_body = edu_response.into_string().expect("edu body");
    let edu_json: Value = serde_json::from_str(&edu_body).expect("valid json");
    let education_id = edu_json["body"]["id"].as_i64().expect("education id") as i32;

    // Add active key point
    let active_kp_payload = serde_json::json!({
        "key_point": "Active key point",
        "display_order": 1
    });

    let active_kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(active_kp_payload.to_string())
        .dispatch();

    assert_eq!(active_kp_response.status(), Status::Created);

    let active_kp_body = active_kp_response.into_string().expect("active kp body");
    let active_kp_json: Value = serde_json::from_str(&active_kp_body).expect("valid json");
    let _active_kp_id = active_kp_json["body"]["id"].as_i64().expect("active kp id") as i32;

    // Add inactive key point
    let inactive_kp_payload = serde_json::json!({
        "key_point": "Inactive key point",
        "display_order": 2
    });

    let inactive_kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(inactive_kp_payload.to_string())
        .dispatch();

    assert_eq!(inactive_kp_response.status(), Status::Created);

    let inactive_kp_body = inactive_kp_response
        .into_string()
        .expect("inactive kp body");
    let inactive_kp_json: Value = serde_json::from_str(&inactive_kp_body).expect("valid json");
    let inactive_kp_id = inactive_kp_json["body"]["id"]
        .as_i64()
        .expect("inactive kp id") as i32;

    // Deactivate the second key point
    let deactivate_payload = serde_json::json!({
        "active": false
    });

    let deactivate_response = fixture
        .client()
        .put(format!("/api/education_key_points/{}", inactive_kp_id))
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(deactivate_payload.to_string())
        .dispatch();

    assert_eq!(deactivate_response.status(), Status::Ok);

    // Owner should see both key points
    let owner_kps_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(support::auth_header(&owner.token))
        .dispatch();

    assert_eq!(owner_kps_response.status(), Status::Ok);
    let owner_kps_body = owner_kps_response.into_string().expect("owner kps body");
    let owner_kps_json: Value = serde_json::from_str(&owner_kps_body).expect("valid json");
    let owner_kps = owner_kps_json["body"].as_array().expect("owner kps array");
    assert_eq!(owner_kps.len(), 2);

    // Non-owner should see only active key points
    let non_owner_kps_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(support::auth_header(&non_owner.token))
        .dispatch();

    assert_eq!(non_owner_kps_response.status(), Status::Ok);
    let non_owner_kps_body = non_owner_kps_response
        .into_string()
        .expect("non-owner kps body");
    let non_owner_kps_json: Value = serde_json::from_str(&non_owner_kps_body).expect("valid json");
    let non_owner_kps = non_owner_kps_json["body"]
        .as_array()
        .expect("non-owner kps array");
    assert_eq!(non_owner_kps.len(), 1);
    assert_eq!(non_owner_kps[0]["key_point"], "Active key point");

    // Anonymous user should also see only active key points
    let anon_kps_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .dispatch();

    assert_eq!(anon_kps_response.status(), Status::Ok);
    let anon_kps_body = anon_kps_response.into_string().expect("anon kps body");
    let anon_kps_json: Value = serde_json::from_str(&anon_kps_body).expect("valid json");
    let anon_kps = anon_kps_json["body"].as_array().expect("anon kps array");
    assert_eq!(anon_kps.len(), 1);
    assert_eq!(anon_kps[0]["key_point"], "Active key point");
}
