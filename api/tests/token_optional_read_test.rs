//! Token-optional reads must serve the anonymous surface even when the
//! caller presents a bearer the guard cannot resolve.
//!
//! The regression this pins: a stale `resume_api_token` left in the browser's
//! `localStorage` made every mount-time read answer 401 until the page was
//! reloaded. The bearer only ever widens a response, so an unresolvable one
//! must not deny the anonymous read it was already entitled to.
//!
//! Strict routes are unaffected and keep answering 401.

use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::LocalResponse;
use serde_json::Value;

mod support;

fn bearer(value: &str) -> Header<'static> {
    Header::new("Authorization", format!("Bearer {}", value))
}

fn body_json(response: LocalResponse<'_>) -> Value {
    let text = response.into_string().expect("response body");
    serde_json::from_str(&text).expect("valid json body")
}

/// The three bearer shapes that resolve to no unexpired session row.
fn unresolvable_bearers(expired_token: &str) -> Vec<(&'static str, String)> {
    vec![
        ("expired session row", expired_token.to_string()),
        ("valid uuid with no row", uuid::Uuid::new_v4().to_string()),
        ("malformed bearer", "not-a-uuid".to_string()),
    ]
}

/// A resume plus every JSON read route that reaches it.
struct Seeded {
    resume_id: uuid::Uuid,
    routes: Vec<(&'static str, String)>,
}

fn create_child(fixture: &support::Fixture, token: &str, path: &str, payload: Value) -> i64 {
    let response = fixture
        .client()
        .post(path)
        .header(support::auth_header(token))
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Created, "create {path} failed");
    body_json(response)["body"]["id"]
        .as_i64()
        .unwrap_or_else(|| panic!("{path} should return a numeric child id"))
}

/// Seeds a `public` resume owned by `token`'s user with one child in every
/// section, so an anonymous read returns a non-empty, viewer-filtered body
/// rather than two empty lists that would compare equal for free.
fn seed_public_resume(fixture: &mut support::Fixture, token: &str) -> Seeded {
    let create = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(token))
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Anonymous Surface",
                "email": support::unique_email("token.optional.public"),
                "visibility": "public"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(create.status(), Status::Created, "resume create failed");
    let resume_id = support::resume_uuid(&body_json(create)["body"]["id"]);
    fixture.track_resume_id(resume_id);

    create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/skills"),
        serde_json::json!({
            "skill_name": "Rust",
            "confidence_percentage": 90,
            "display_order": 0
        }),
    );

    let education_id = create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/education"),
        serde_json::json!({
            "education_stage": "BSc",
            "institution_name": "Some University",
            "start_date": "2015-09-01",
            "end_date": "2018-06-30",
            "display_order": 0
        }),
    );
    create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/education/{education_id}/key_points"),
        serde_json::json!({"key_point": "Graduated with honours", "display_order": 0}),
    );

    let work_id = create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/work_experiences"),
        serde_json::json!({
            "job_title": "Engineer",
            "company_name": "Some Company",
            "start_date": "2019-01",
            "description": "Built things",
            "display_order": 0
        }),
    );
    create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/work_experiences/{work_id}/key_points"),
        serde_json::json!({"key_point": "Shipped the thing", "display_order": 0}),
    );

    let project_id = create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/portfolio_projects"),
        serde_json::json!({
            "project_name": "Some Project",
            "description": "Did the thing",
            "display_order": 0
        }),
    );
    create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/portfolio_projects/{project_id}/key_points"),
        serde_json::json!({"key_point": "Delivered on time", "display_order": 0}),
    );
    create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/portfolio_projects/{project_id}/technologies"),
        serde_json::json!({"technology_name": "PostgreSQL", "display_order": 0}),
    );

    let language_id = create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/languages"),
        serde_json::json!({"language_name": "English", "display_order": 0}),
    );
    create_child(
        fixture,
        token,
        &format!("/api/resume/{resume_id}/languages/{language_id}/frameworks"),
        serde_json::json!({"framework_name": "Rocket", "display_order": 0}),
    );

    let routes = vec![
        ("read one resume", format!("/api/resume/{resume_id}")),
        ("list skills", format!("/api/resume/{resume_id}/skills")),
        (
            "list education",
            format!("/api/resume/{resume_id}/education"),
        ),
        (
            "list education key points",
            format!("/api/resume/{resume_id}/education/{education_id}/key_points"),
        ),
        (
            "list work experiences",
            format!("/api/resume/{resume_id}/work_experiences"),
        ),
        (
            "list work key points",
            format!("/api/resume/{resume_id}/work_experiences/{work_id}/key_points"),
        ),
        (
            "list portfolio projects",
            format!("/api/resume/{resume_id}/portfolio_projects"),
        ),
        (
            "list portfolio key points",
            format!("/api/resume/{resume_id}/portfolio_projects/{project_id}/key_points"),
        ),
        (
            "list portfolio technologies",
            format!("/api/resume/{resume_id}/portfolio_projects/{project_id}/technologies"),
        ),
        (
            "list languages",
            format!("/api/resume/{resume_id}/languages"),
        ),
        (
            "list frameworks",
            format!("/api/resume/{resume_id}/languages/{language_id}/frameworks"),
        ),
        ("list variants", format!("/api/resume/{resume_id}/variants")),
    ];

    Seeded { resume_id, routes }
}

/// Every JSON token-optional read must answer 200 with a body identical to
/// the anonymous body, for each unresolvable bearer shape.
fn assert_anonymous_json_surface(
    fixture: &support::Fixture,
    routes: &[(&str, String)],
    expired: &str,
) {
    for (label, path) in routes {
        let anonymous = fixture.client().get(path).dispatch();
        assert_eq!(
            anonymous.status(),
            Status::Ok,
            "{label}: anonymous read should be 200"
        );
        let anonymous_body = body_json(anonymous);

        for (shape, token) in unresolvable_bearers(expired) {
            let response = fixture.client().get(path).header(bearer(&token)).dispatch();

            assert_eq!(
                response.status(),
                Status::Ok,
                "{label} with a {shape} should read as anonymous, not 401"
            );
            assert_eq!(
                body_json(response),
                anonymous_body,
                "{label} with a {shape} should match the anonymous body exactly"
            );
        }
    }
}

/// The listing route answers with every readable resume, not only this
/// fixture's, so it is compared entry by entry. Comparing the whole body
/// twice would race with a sibling test that creates its own public resume
/// in between.
fn assert_anonymous_listing(fixture: &support::Fixture, resume_id: uuid::Uuid, expired: &str) {
    let entry = |body: &Value| -> Value {
        body["body"]["items"]
            .as_array()
            .expect("listing body should be an array")
            .iter()
            .find(|row| row["id"] == resume_id.to_string())
            .cloned()
            .expect("the fixture's own resume should be listed")
    };

    let anonymous = fixture.client().get("/api/resumes").dispatch();
    assert_eq!(
        anonymous.status(),
        Status::Ok,
        "anonymous listing should be 200"
    );
    let anonymous_entry = entry(&body_json(anonymous));

    for (shape, token) in unresolvable_bearers(expired) {
        let response = fixture
            .client()
            .get("/api/resumes")
            .header(bearer(&token))
            .dispatch();

        assert_eq!(
            response.status(),
            Status::Ok,
            "listing with a {shape} should read as anonymous, not 401"
        );
        assert_eq!(
            entry(&body_json(response)),
            anonymous_entry,
            "listing with a {shape} should match the anonymous entry exactly"
        );
    }
}

#[test]
fn unresolvable_bearer_reads_as_anonymous_on_every_token_optional_route() {
    let mut fixture = support::Fixture::new(9_242_001);

    let owner = support::register_and_login(fixture.client(), "token.optional.owner");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let seeded = seed_public_resume(&mut fixture, &owner.token);

    fixture.expire_session(&owner.token);

    assert_anonymous_listing(&fixture, seeded.resume_id, &owner.token);
    assert_anonymous_json_surface(&fixture, &seeded.routes, &owner.token);
}

#[test]
fn unresolvable_bearer_reads_as_anonymous_on_markdown_export() {
    let mut fixture = support::Fixture::new(9_242_002);

    let owner = support::register_and_login(fixture.client(), "token.optional.export");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let seeded = seed_public_resume(&mut fixture, &owner.token);
    let path = format!("/api/resume/{}/export/markdown", seeded.resume_id);

    fixture.expire_session(&owner.token);

    let anonymous = fixture.client().get(&path).dispatch();
    assert_eq!(
        anonymous.status(),
        Status::Ok,
        "anonymous export should be 200"
    );
    let anonymous_body = anonymous.into_string().expect("markdown body");

    for (shape, token) in unresolvable_bearers(&owner.token) {
        let response = fixture
            .client()
            .get(&path)
            .header(bearer(&token))
            .dispatch();

        assert_eq!(
            response.status(),
            Status::Ok,
            "markdown export with a {shape} should read as anonymous, not 401"
        );
        assert_eq!(
            response.into_string().expect("markdown body"),
            anonymous_body,
            "markdown export with a {shape} should match the anonymous body exactly"
        );
    }
}

#[test]
fn lowercase_bearer_prefix_still_authenticates_as_owner() {
    let mut fixture = support::Fixture::new(9_242_003);

    let owner = support::register_and_login(fixture.client(), "token.optional.lowercase");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let private_email = support::unique_email("token.optional.private");
    let create = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Owner Only",
                "email": private_email,
                "visibility": "private"
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(create.status(), Status::Created);
    let private_resume_id = support::resume_uuid(&body_json(create)["body"]["id"]);
    fixture.track_resume_id(private_resume_id);

    let lowercase = Header::new("Authorization", format!("bearer {}", owner.token));

    let listed = fixture
        .client()
        .get("/api/resumes")
        .header(lowercase.clone())
        .dispatch();
    assert_eq!(listed.status(), Status::Ok);
    let listed_body = body_json(listed);
    assert!(
        listed_body["body"]["items"]
            .as_array()
            .expect("array")
            .iter()
            .any(|row| row["id"].as_str() == Some(private_resume_id.to_string().as_str())),
        "a lowercase `bearer` prefix must still authenticate as the owner"
    );

    let read = fixture
        .client()
        .get(format!("/api/resume/{private_resume_id}"))
        .header(lowercase)
        .dispatch();
    assert_eq!(
        read.status(),
        Status::Ok,
        "a lowercase `bearer` prefix must still read a private resume as its owner"
    );
}

#[test]
fn private_resume_with_an_unresolvable_bearer_reads_as_absent() {
    let mut fixture = support::Fixture::new(9_242_004);

    let owner = support::register_and_login(fixture.client(), "token.optional.hidden");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let create = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Hidden",
                "email": support::unique_email("token.optional.hidden.resume"),
                "visibility": "private"
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(create.status(), Status::Created);
    let private_resume_id = support::resume_uuid(&body_json(create)["body"]["id"]);
    fixture.track_resume_id(private_resume_id);

    fixture.expire_session(&owner.token);

    for (shape, token) in unresolvable_bearers(&owner.token) {
        let response = fixture
            .client()
            .get(format!("/api/resume/{private_resume_id}"))
            .header(bearer(&token))
            .dispatch();

        assert_eq!(
            response.status(),
            Status::NotFound,
            "a private resume with a {shape} should read as absent, not 401"
        );
    }
}

#[test]
fn strict_routes_still_reject_an_unresolvable_bearer() {
    let mut fixture = support::Fixture::new(9_242_005);

    let owner = support::register_and_login(fixture.client(), "token.optional.strict");
    fixture.track_user_id(owner.user_id);
    fixture.track_session_id(owner.token.clone());

    let create = fixture
        .client()
        .post("/api/new_resume")
        .header(support::auth_header(&owner.token))
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Strict Target",
                "email": support::unique_email("token.optional.strict.resume"),
                "visibility": "public"
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(create.status(), Status::Created);
    let resume_id = support::resume_uuid(&body_json(create)["body"]["id"]);
    fixture.track_resume_id(resume_id);

    fixture.expire_session(&owner.token);

    for (shape, token) in unresolvable_bearers(&owner.token) {
        let me = fixture
            .client()
            .get("/api/auth/me")
            .header(bearer(&token))
            .dispatch();
        assert_eq!(
            me.status(),
            Status::Unauthorized,
            "/auth/me with a {shape} must stay strict"
        );

        let write = fixture
            .client()
            .put(format!("/api/resume/{resume_id}"))
            .header(bearer(&token))
            .header(ContentType::JSON)
            .body(serde_json::json!({"location": "Nope"}).to_string())
            .dispatch();
        assert_eq!(
            write.status(),
            Status::Unauthorized,
            "a write with a {shape} must stay strict"
        );

        let import = fixture
            .client()
            .post("/api/resume/import/markdown")
            .header(bearer(&token))
            .header(Header::new("Content-Type", "text/markdown"))
            .body("# Strict Import\n")
            .dispatch();
        assert_eq!(
            import.status(),
            Status::Unauthorized,
            "markdown import with a {shape} must stay strict"
        );
    }
}
