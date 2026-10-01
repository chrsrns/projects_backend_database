use application::error::ApplicationError;
use application::resume::common::app_err_from_diesel_err;
use diesel::prelude::*;
use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn create_base(fixture: &mut support::Fixture, email: &str) -> i32 {
    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Integrity Base",
                "email": email,
                "is_public": true
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Created, "base create");
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let id = json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(id);
    id
}

fn post_variant(fixture: &support::Fixture, base_id: i32) -> (Status, Value) {
    let response = fixture
        .client()
        .post(format!("/api/resume/{}/variants", base_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "company_name": "Acme Corp" }).to_string())
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

#[test]
fn variant_email_shares_base_email_partial_unique_index() {
    let mut fixture = support::Fixture::new(9_241_045);
    let email = format!(
        "variant.unique.{}.{}@example.com",
        9_241_045,
        unique_suffix()
    );
    let base_id = create_base(&mut fixture, &email);

    let duplicate_status = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Duplicate Base",
                "email": email,
                "is_public": true
            })
            .to_string(),
        )
        .dispatch()
        .status();
    assert_eq!(
        duplicate_status,
        Status::Conflict,
        "base resumes still own their email"
    );

    let (status, first) = post_variant(&fixture, base_id);
    assert_eq!(status, Status::Created);
    let first_id = first["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(first_id);

    let (status, second) = post_variant(&fixture, base_id);
    assert_eq!(
        status,
        Status::Created,
        "two variants may share the base email"
    );
    let second_id = second["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(second_id);

    assert_eq!(first["body"]["email"], email);
    assert_eq!(second["body"]["email"], email);
}

#[test]
fn user_delete_cascade_removes_base_and_variant() {
    let mut fixture = support::Fixture::new(9_241_044);
    let email = format!(
        "variant.cascade.{}.{}@example.com",
        9_241_044,
        unique_suffix()
    );
    let base_id = create_base(&mut fixture, &email);

    let (status, variant) = post_variant(&fixture, base_id);
    assert_eq!(status, Status::Created);
    let variant_id = variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    // Removing the owner in one statement must take the base and its
    // variant together; the base foreign key only forbids deleting a base
    // while variants survive it.
    let mut connection = infrastructure::establish_connection();
    diesel::sql_query("DELETE FROM users WHERE id = $1")
        .bind::<diesel::sql_types::Integer, _>(fixture.user_id())
        .execute(&mut connection)
        .expect("the cascade must not be blocked by the variant reference");

    for resume_id in [base_id, variant_id] {
        let rows: Vec<IdRow> = diesel::sql_query("SELECT id FROM resumes WHERE id = $1")
            .bind::<diesel::sql_types::Integer, _>(resume_id)
            .load(&mut connection)
            .expect("probe resume");

        assert!(
            rows.is_empty(),
            "resume {} should have been removed with its owner, still found {:?}",
            resume_id,
            rows.iter().map(|row| row.id).collect::<Vec<_>>()
        );
    }

    fixture.untrack_resume_id(base_id);
    fixture.untrack_resume_id(variant_id);
}

#[test]
fn fk_violation_maps_to_409() {
    let _fixture = support::Fixture::new(9_241_043);
    let mut connection = infrastructure::establish_connection();

    let error = diesel::sql_query(
        "INSERT INTO resumes (name, email, is_public, base_resume_id) VALUES ($1, $2, TRUE, $3)",
    )
    .bind::<diesel::sql_types::Text, _>("Orphan Variant")
    .bind::<diesel::sql_types::Text, _>(format!("variant.orphan.{}@example.com", unique_suffix()))
    .bind::<diesel::sql_types::Integer, _>(2_000_000_000)
    .execute(&mut connection)
    .expect_err("a base reference to a missing resume must be rejected");

    assert_eq!(
        app_err_from_diesel_err(error),
        ApplicationError::Conflict("Foreign key violation".to_string()),
        "the API answers 409 for a reference failure"
    );
}
