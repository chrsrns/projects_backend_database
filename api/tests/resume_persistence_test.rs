use diesel::pg::PgConnection;
use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use std::time::Duration;

/// Test fixture that automatically cleans up created resumes
struct TestFixture {
    client: Client,
    created_resume_ids: Vec<i32>,
    auth_token: String,
    lock_key: i64,
    lock_connection: PgConnection,
}

impl TestFixture {
    fn new() -> Self {
        let lock_key: i64 = 9_225_337;
        let mut lock_connection = establish_connection();
        diesel::sql_query(format!("SELECT pg_advisory_lock({})", lock_key))
            .execute(&mut lock_connection)
            .expect("acquire advisory lock");

        let rocket = api::build_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        // Create a dedicated user for this fixture and login to get a Bearer token.
        // Note: responses borrow the client, so compute token in a scope that ends
        // before we move the client into the fixture.
        let auth_token = {
            let unique_email = format!(
                "resume.tests.{}@example.com",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );

            let register_payload = serde_json::json!({
                "email": unique_email,
                "password": "test-password"
            });

            let register_response = client
                .post("/api/auth/register")
                .header(ContentType::JSON)
                .body(register_payload.to_string())
                .dispatch();
            assert_eq!(register_response.status(), Status::Created);

            let login_payload = serde_json::json!({
                "email": register_payload["email"].as_str().unwrap(),
                "password": "test-password"
            });

            let login_response = client
                .post("/api/auth/login")
                .header(ContentType::JSON)
                .body(login_payload.to_string())
                .dispatch();
            assert_eq!(login_response.status(), Status::Ok);

            let login_body = login_response.into_string().expect("login body");
            let login_json: Value = serde_json::from_str(&login_body).expect("valid json");
            login_json["body"]["AuthToken"]["token"]
                .as_str()
                .expect("token")
                .to_string()
        };

        TestFixture {
            client,
            created_resume_ids: Vec::new(),
            auth_token,
            lock_key,
            lock_connection,
        }
    }

    fn client(&self) -> &Client {
        &self.client
    }

    fn auth_header(&self) -> Header<'static> {
        Header::new("Authorization", format!("Bearer {}", self.auth_token))
    }

    fn track_resume_id(&mut self, id: i32) {
        self.created_resume_ids.push(id);
        println!("Tracking resume ID {} for cleanup", id);
    }

    fn untrack_resume_id(&mut self, id: i32) {
        self.created_resume_ids.retain(|&existing| existing != id);
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        if !self.created_resume_ids.is_empty() {
            println!("Cleaning up {} resume(s)...", self.created_resume_ids.len());

            use domain::schema::resumes::dsl::*;
            let connection = &mut self.lock_connection;

            for resume_id in &self.created_resume_ids {
                match diesel::delete(resumes.find(resume_id)).execute(connection) {
                    Ok(_) => println!("✓ Deleted resume ID {}", resume_id),
                    Err(e) => eprintln!("✗ Failed to delete resume ID {}: {}", resume_id, e),
                }
            }

            println!("Cleanup complete!");
        }

        diesel::sql_query(format!("SELECT pg_advisory_unlock({})", self.lock_key))
            .execute(&mut self.lock_connection)
            .expect("release advisory lock");
    }
}

#[test]
fn test_create_and_retrieve_resume_persistence() {
    let mut fixture = TestFixture::new();

    // Test data to create
    let new_resume_json = serde_json::json!({
        "name": "John Doe",
        "profile_image_url": "https://example.com/profile.jpg",
        "location": "San Francisco, CA",
        "email": "john.doe@example.com",
        "github_url": "https://github.com/johndoe",
        "mobile_number": "+1234567890",
        "is_public": true
    });

    // Step 1: Create a new resume via POST
    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    // Verify creation was successful
    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");

    // Extract the created resume from response
    let created_resume = &create_json["body"]["Resume"];
    let resume_id = created_resume["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;

    // Track for cleanup
    fixture.track_resume_id(resume_id);

    println!("Created resume with ID: {}", resume_id);

    // Step 2: Retrieve the resume via GET to verify persistence
    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    // Verify retrieval was successful
    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("Response body");
    let get_json: Value = serde_json::from_str(&get_body).expect("Valid JSON");

    let retrieved_resume = &get_json["body"]["Resume"];

    // Step 3: Verify all fields match
    assert_eq!(retrieved_resume["id"], resume_id);
    assert_eq!(retrieved_resume["name"], "John Doe");
    assert_eq!(
        retrieved_resume["profile_image_url"],
        "https://example.com/profile.jpg"
    );
    assert_eq!(retrieved_resume["location"], "San Francisco, CA");
    assert_eq!(retrieved_resume["email"], "john.doe@example.com");
    assert_eq!(retrieved_resume["github_url"], "https://github.com/johndoe");
    assert_eq!(retrieved_resume["mobile_number"], "+1234567890");

    // Verify timestamp fields exist
    assert!(
        retrieved_resume["created_at"].is_string(),
        "created_at should be present"
    );
    assert!(
        retrieved_resume["updated_at"].is_string(),
        "updated_at should be present"
    );

    println!("✓ Resume persistence verified successfully!");
}

#[test]
fn test_create_resume_appears_in_list() {
    let mut fixture = TestFixture::new();

    // Create a new resume with unique email to avoid conflicts
    let unique_email = format!(
        "jane.smith.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Jane Smith",
        "profile_image_url": null,
        "location": "New York, NY",
        "email": unique_email,
        "github_url": "https://github.com/janesmith",
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

    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let created_resume = &create_json["body"]["Resume"];
    let created_id = created_resume["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;

    // Track for cleanup
    fixture.track_resume_id(created_id);

    println!(
        "Created resume with ID: {} and email: {}",
        created_id, unique_email
    );

    // Verify the resume appears in the list
    let list_response = fixture.client().get("/api/resumes").dispatch();
    assert_eq!(list_response.status(), Status::Ok);
    let list_body = list_response.into_string().expect("Response body");
    let list_json: Value = serde_json::from_str(&list_body).expect("Valid JSON");

    let resumes_array = list_json["body"]["Resumes"]
        .as_array()
        .expect("Should be array");

    println!("Total resumes in list: {}", resumes_array.len());

    // Verify the specific resume we created is in the list
    let found = resumes_array
        .iter()
        .any(|r| r["email"] == unique_email && r["id"] == created_id);

    assert!(
        found,
        "Newly created resume with ID {} and email {} should appear in the list",
        created_id, unique_email
    );

    // Also verify all fields are correct in the list
    let created_resume_in_list = resumes_array
        .iter()
        .find(|r| r["id"] == created_id)
        .expect("Should find the created resume");

    assert_eq!(created_resume_in_list["name"], "Jane Smith");
    assert_eq!(created_resume_in_list["location"], "New York, NY");
    assert_eq!(created_resume_in_list["email"], unique_email);
    assert_eq!(
        created_resume_in_list["github_url"],
        "https://github.com/janesmith"
    );

    println!("✓ Resume appears in list after creation with all correct fields!");
}

#[test]
fn test_retrieve_nonexistent_resume() {
    let fixture = TestFixture::new();

    // Try to retrieve a resume with an ID that doesn't exist
    let nonexistent_id = 999999;
    let response = fixture
        .client()
        .get(format!("/api/resume/{}", nonexistent_id))
        .dispatch();

    // Should return 404 Not Found
    assert_eq!(response.status(), Status::NotFound);

    let body = response.into_string().expect("Response body");
    let json: Value = serde_json::from_str(&body).expect("Valid JSON");

    // Verify error message contains information about the missing ID
    let message = json["body"]["Message"]
        .as_str()
        .expect("Should have message");
    assert!(
        message.contains(&nonexistent_id.to_string()),
        "Error message should mention the resume ID"
    );

    println!("✓ Nonexistent resume returns proper 404 error!");
}

#[test]
fn test_update_and_retrieve_resume_persistence() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "update.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Update User",
        "profile_image_url": null,
        "location": "Old Location",
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

    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let created_resume = &create_json["body"]["Resume"];
    let resume_id = created_resume["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;

    fixture.track_resume_id(resume_id);

    let update_resume_json = serde_json::json!({
        "location": "New Location"
    });

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_resume_json.to_string())
        .dispatch();

    assert_eq!(update_response.status(), Status::Ok);

    let update_body = update_response.into_string().expect("Response body");
    let update_json: Value = serde_json::from_str(&update_body).expect("Valid JSON");
    let updated_resume = &update_json["body"]["Resume"];
    assert_eq!(updated_resume["id"], resume_id);
    assert_eq!(updated_resume["location"], "New Location");

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::Ok);

    let get_body = get_response.into_string().expect("Response body");
    let get_json: Value = serde_json::from_str(&get_body).expect("Valid JSON");
    let retrieved_resume = &get_json["body"]["Resume"];
    assert_eq!(retrieved_resume["id"], resume_id);
    assert_eq!(retrieved_resume["location"], "New Location");
}

#[test]
fn test_update_nonexistent_resume() {
    let fixture = TestFixture::new();

    let nonexistent_id = 999999;
    let update_resume_json = serde_json::json!({
        "location": "Does Not Matter"
    });

    let response = fixture
        .client()
        .put(format!("/api/resume/{}", nonexistent_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_resume_json.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body = response.into_string().expect("Response body");
    let json: Value = serde_json::from_str(&body).expect("Valid JSON");
    let message = json["body"]["Message"]
        .as_str()
        .expect("Should have message");
    assert!(message.contains(&nonexistent_id.to_string()));
}

#[test]
fn test_delete_resume_then_not_found() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "delete.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Delete User",
        "profile_image_url": null,
        "location": "Delete Location",
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

    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let created_resume = &create_json["body"]["Resume"];
    let resume_id = created_resume["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;

    fixture.track_resume_id(resume_id);

    {
        let delete_response = fixture
            .client()
            .delete(format!("/api/resume/{}", resume_id))
            .header(fixture.auth_header())
            .dispatch();

        assert_eq!(delete_response.status(), Status::NoContent);
    }

    fixture.untrack_resume_id(resume_id);

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .dispatch();

    assert_eq!(get_response.status(), Status::NotFound);
}

#[test]
fn test_delete_nonexistent_resume() {
    let fixture = TestFixture::new();

    let nonexistent_id = 999999;
    let response = fixture
        .client()
        .delete(format!("/api/resume/{}", nonexistent_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body = response.into_string().expect("Response body");
    let json: Value = serde_json::from_str(&body).expect("Valid JSON");
    let message = json["body"]["Message"]
        .as_str()
        .expect("Should have message");
    assert!(message.contains(&nonexistent_id.to_string()));
}

#[test]
fn test_create_resume_missing_required_fields_returns_422() {
    let fixture = TestFixture::new();

    let invalid_json_missing_email = serde_json::json!({
        "name": "Missing Email"
    });

    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(invalid_json_missing_email.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
}

#[test]
fn test_create_resume_duplicate_email_returns_409() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "dupe.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Dupe User",
        "profile_image_url": null,
        "location": "Dupe Location",
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": true
    });

    let first_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(first_response.status(), Status::Created);

    let first_body = first_response.into_string().expect("Response body");
    let first_json: Value = serde_json::from_str(&first_body).expect("Valid JSON");
    let created_resume = &first_json["body"]["Resume"];
    let resume_id = created_resume["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;
    fixture.track_resume_id(resume_id);

    let second_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(second_response.status(), Status::Conflict);

    let second_body = second_response.into_string().expect("Response body");
    let second_json: Value = serde_json::from_str(&second_body).expect("Valid JSON");
    assert!(second_json["body"]["Message"].is_string());
}

#[test]
fn test_list_resumes_ordering_is_deterministic_for_created_records() {
    let mut fixture = TestFixture::new();

    let email_a = format!(
        "order.a.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let email_b = format!(
        "order.b.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let resume_a = serde_json::json!({
        "name": "Order A",
        "profile_image_url": null,
        "location": "Loc A",
        "email": email_a,
        "github_url": null,
        "mobile_number": null,
        "is_public": true
    });

    let resume_b = serde_json::json!({
        "name": "Order B",
        "profile_image_url": null,
        "location": "Loc B",
        "email": email_b,
        "github_url": null,
        "mobile_number": null,
        "is_public": true
    });

    let create_a = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(resume_a.to_string())
        .dispatch();
    assert_eq!(create_a.status(), Status::Created);
    let create_a_body = create_a.into_string().expect("Response body");
    let create_a_json: Value = serde_json::from_str(&create_a_body).expect("Valid JSON");
    let id_a = create_a_json["body"]["Resume"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(id_a);

    let create_b = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(resume_b.to_string())
        .dispatch();
    assert_eq!(create_b.status(), Status::Created);
    let create_b_body = create_b.into_string().expect("Response body");
    let create_b_json: Value = serde_json::from_str(&create_b_body).expect("Valid JSON");
    let id_b = create_b_json["body"]["Resume"]["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(id_b);

    let list_response = fixture.client().get("/api/resumes").dispatch();
    assert_eq!(list_response.status(), Status::Ok);
    let list_body = list_response.into_string().expect("Response body");
    let list_json: Value = serde_json::from_str(&list_body).expect("Valid JSON");

    let resumes_array = list_json["body"]["Resumes"]
        .as_array()
        .expect("Should be array");

    let our_ids: Vec<i32> = resumes_array
        .iter()
        .filter_map(|r| {
            let id = r["id"].as_i64()? as i32;
            if id == id_a || id == id_b {
                Some(id)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        our_ids.len(),
        2,
        "Both created resumes should appear in list"
    );

    let expected = if id_a < id_b {
        vec![id_a, id_b]
    } else {
        vec![id_b, id_a]
    };
    assert_eq!(
        our_ids, expected,
        "List order should be deterministic (sorted by Resume Ord)"
    );
}

#[test]
fn test_updated_at_changes_on_update_created_at_stays_same() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "timestamp.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Timestamp User",
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
    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let created = &create_json["body"]["Resume"];

    let resume_id = created["id"].as_i64().expect("id") as i32;
    fixture.track_resume_id(resume_id);

    let created_at_before = created["created_at"]
        .as_str()
        .expect("created_at string")
        .to_string();
    let updated_at_before = created["updated_at"]
        .as_str()
        .expect("updated_at string")
        .to_string();

    std::thread::sleep(Duration::from_secs(1));

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
    let update_body = update_response.into_string().expect("Response body");
    let update_json: Value = serde_json::from_str(&update_body).expect("Valid JSON");
    let updated = &update_json["body"]["Resume"];

    let created_at_after = updated["created_at"].as_str().expect("created_at string");
    let updated_at_after = updated["updated_at"].as_str().expect("updated_at string");

    assert_eq!(
        created_at_after, created_at_before,
        "created_at should not change on update"
    );
    assert_ne!(
        updated_at_after, updated_at_before,
        "updated_at should change on update"
    );
}

#[test]
fn test_skills_crud_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "skills.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Skills User",
        "profile_image_url": null,
        "location": null,
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_resume_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_resume_response.status(), Status::Created);
    let create_resume_body = create_resume_response
        .into_string()
        .expect("create resume body");
    let create_resume_json: Value = serde_json::from_str(&create_resume_body).expect("valid json");
    let resume_id = create_resume_json["body"]["Resume"]["id"]
        .as_i64()
        .expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

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
    let create_skill_body = create_skill_response
        .into_string()
        .expect("create skill body");
    let create_skill_json: Value = serde_json::from_str(&create_skill_body).expect("valid json");
    let skill_id = create_skill_json["body"]["Skill"]["id"]
        .as_i64()
        .expect("skill id") as i32;
    assert_eq!(
        create_skill_json["body"]["Skill"]["resume_id"]
            .as_i64()
            .expect("resume_id") as i32,
        resume_id
    );
    assert_eq!(
        create_skill_json["body"]["Skill"]["skill_name"]
            .as_str()
            .expect("skill_name"),
        "Rust"
    );
    assert_eq!(
        create_skill_json["body"]["Skill"]["confidence_percentage"]
            .as_i64()
            .expect("confidence_percentage") as i32,
        80
    );

    let list_skills_response = fixture
        .client()
        .get(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_skills_response.status(), Status::Ok);
    let list_skills_body = list_skills_response
        .into_string()
        .expect("list skills body");
    let list_skills_json: Value = serde_json::from_str(&list_skills_body).expect("valid json");
    let skills_array = list_skills_json["body"]["Skills"]
        .as_array()
        .expect("skills array");
    assert!(
        skills_array
            .iter()
            .any(|s| s["id"].as_i64() == Some(skill_id as i64))
    );

    let update_skill_json = serde_json::json!({
        "confidence_percentage": 95
    });

    let update_skill_response = fixture
        .client()
        .put(format!("/api/skills/{}", skill_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_skill_json.to_string())
        .dispatch();

    assert_eq!(update_skill_response.status(), Status::Ok);
    let update_skill_body = update_skill_response
        .into_string()
        .expect("update skill body");
    let update_skill_json: Value = serde_json::from_str(&update_skill_body).expect("valid json");
    assert_eq!(
        update_skill_json["body"]["Skill"]["id"]
            .as_i64()
            .expect("id") as i32,
        skill_id
    );
    assert_eq!(
        update_skill_json["body"]["Skill"]["confidence_percentage"]
            .as_i64()
            .expect("confidence_percentage") as i32,
        95
    );

    let delete_skill_response = fixture
        .client()
        .delete(format!("/api/skills/{}", skill_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(delete_skill_response.status(), Status::NoContent);

    let list_after_delete_response = fixture
        .client()
        .get(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_after_delete_response.status(), Status::Ok);
    let list_after_delete_body = list_after_delete_response
        .into_string()
        .expect("list skills after delete body");
    let list_after_delete_json: Value =
        serde_json::from_str(&list_after_delete_body).expect("valid json");
    let skills_after_delete = list_after_delete_json["body"]["Skills"]
        .as_array()
        .expect("skills array");
    assert!(
        !skills_after_delete
            .iter()
            .any(|s| s["id"].as_i64() == Some(skill_id as i64))
    );
}

#[test]
fn test_languages_crud_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "languages.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Languages User",
        "profile_image_url": null,
        "location": null,
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_resume_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_resume_response.status(), Status::Created);
    let create_resume_body = create_resume_response
        .into_string()
        .expect("create resume body");
    let create_resume_json: Value = serde_json::from_str(&create_resume_body).expect("valid json");
    let resume_id = create_resume_json["body"]["Resume"]["id"]
        .as_i64()
        .expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let new_language_json = serde_json::json!({
        "language_name": "TypeScript",
        "display_order": 0
    });

    let create_language_response = fixture
        .client()
        .post(format!("/api/resume/{}/languages", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_language_json.to_string())
        .dispatch();

    assert_eq!(create_language_response.status(), Status::Created);
    let create_language_body = create_language_response
        .into_string()
        .expect("create language body");
    let create_language_json: Value =
        serde_json::from_str(&create_language_body).expect("valid json");
    let language_id = create_language_json["body"]["Language"]["id"]
        .as_i64()
        .expect("language id") as i32;
    assert_eq!(
        create_language_json["body"]["Language"]["resume_id"]
            .as_i64()
            .expect("resume_id") as i32,
        resume_id
    );
    assert_eq!(
        create_language_json["body"]["Language"]["language_name"]
            .as_str()
            .expect("language_name"),
        "TypeScript"
    );

    let list_languages_response = fixture
        .client()
        .get(format!("/api/resume/{}/languages", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_languages_response.status(), Status::Ok);
    let list_languages_body = list_languages_response
        .into_string()
        .expect("list languages body");
    let list_languages_json: Value =
        serde_json::from_str(&list_languages_body).expect("valid json");
    let languages_array = list_languages_json["body"]["Languages"]
        .as_array()
        .expect("languages array");
    assert!(
        languages_array
            .iter()
            .any(|l| l["id"].as_i64() == Some(language_id as i64))
    );

    let update_language_json = serde_json::json!({
        "language_name": "JavaScript"
    });

    let update_language_response = fixture
        .client()
        .put(format!("/api/languages/{}", language_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_language_json.to_string())
        .dispatch();

    assert_eq!(update_language_response.status(), Status::Ok);
    let update_language_body = update_language_response
        .into_string()
        .expect("update language body");
    let update_language_json: Value =
        serde_json::from_str(&update_language_body).expect("valid json");
    assert_eq!(
        update_language_json["body"]["Language"]["id"]
            .as_i64()
            .expect("id") as i32,
        language_id
    );
    assert_eq!(
        update_language_json["body"]["Language"]["language_name"]
            .as_str()
            .expect("language_name"),
        "JavaScript"
    );

    let delete_language_response = fixture
        .client()
        .delete(format!("/api/languages/{}", language_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(delete_language_response.status(), Status::NoContent);
}

#[test]
fn test_frameworks_crud_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "frameworks.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Frameworks User",
        "profile_image_url": null,
        "location": null,
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_resume_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_resume_response.status(), Status::Created);
    let create_resume_body = create_resume_response
        .into_string()
        .expect("create resume body");
    let create_resume_json: Value = serde_json::from_str(&create_resume_body).expect("valid json");
    let resume_id = create_resume_json["body"]["Resume"]["id"]
        .as_i64()
        .expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let new_language_json = serde_json::json!({
        "language_name": "Rust",
        "display_order": 0
    });

    let create_language_response = fixture
        .client()
        .post(format!("/api/resume/{}/languages", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_language_json.to_string())
        .dispatch();

    assert_eq!(create_language_response.status(), Status::Created);
    let create_language_body = create_language_response
        .into_string()
        .expect("create language body");
    let create_language_json: Value =
        serde_json::from_str(&create_language_body).expect("valid json");
    let language_id = create_language_json["body"]["Language"]["id"]
        .as_i64()
        .expect("language id") as i32;

    let new_framework_json = serde_json::json!({
        "framework_name": "Rocket",
        "display_order": 0
    });

    let create_framework_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/languages/{}/frameworks",
            resume_id, language_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_framework_json.to_string())
        .dispatch();

    assert_eq!(create_framework_response.status(), Status::Created);
    let create_framework_body = create_framework_response
        .into_string()
        .expect("create framework body");
    let create_framework_json: Value =
        serde_json::from_str(&create_framework_body).expect("valid json");
    let framework_id = create_framework_json["body"]["Framework"]["id"]
        .as_i64()
        .expect("framework id") as i32;
    assert_eq!(
        create_framework_json["body"]["Framework"]["language_id"]
            .as_i64()
            .expect("language_id") as i32,
        language_id
    );
    assert_eq!(
        create_framework_json["body"]["Framework"]["framework_name"]
            .as_str()
            .expect("framework_name"),
        "Rocket"
    );

    let list_frameworks_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/languages/{}/frameworks",
            resume_id, language_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_frameworks_response.status(), Status::Ok);
    let list_frameworks_body = list_frameworks_response
        .into_string()
        .expect("list frameworks body");
    let list_frameworks_json: Value =
        serde_json::from_str(&list_frameworks_body).expect("valid json");
    let frameworks_array = list_frameworks_json["body"]["Frameworks"]
        .as_array()
        .expect("frameworks array");
    assert!(
        frameworks_array
            .iter()
            .any(|f| f["id"].as_i64() == Some(framework_id as i64))
    );

    let update_framework_json = serde_json::json!({
        "framework_name": "Diesel",
    });

    let update_framework_response = fixture
        .client()
        .put(format!("/api/frameworks/{}", framework_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_framework_json.to_string())
        .dispatch();

    assert_eq!(update_framework_response.status(), Status::Ok);
    let update_framework_body = update_framework_response
        .into_string()
        .expect("update framework body");
    let update_framework_json: Value =
        serde_json::from_str(&update_framework_body).expect("valid json");
    assert_eq!(
        update_framework_json["body"]["Framework"]["id"]
            .as_i64()
            .expect("id") as i32,
        framework_id
    );
    assert_eq!(
        update_framework_json["body"]["Framework"]["framework_name"]
            .as_str()
            .expect("framework_name"),
        "Diesel"
    );

    let delete_framework_response = fixture
        .client()
        .delete(format!("/api/frameworks/{}", framework_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(delete_framework_response.status(), Status::NoContent);
}

#[test]
fn test_education_and_key_points_crud_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "education.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Education User",
        "profile_image_url": null,
        "location": null,
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_resume_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_resume_response.status(), Status::Created);
    let create_resume_body = create_resume_response
        .into_string()
        .expect("create resume body");
    let create_resume_json: Value = serde_json::from_str(&create_resume_body).expect("valid json");
    let resume_id = create_resume_json["body"]["Resume"]["id"]
        .as_i64()
        .expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let new_education_json = serde_json::json!({
        "education_stage": "University",
        "institution_name": "Example University",
        "degree": "BSc Computer Science",
        "start_date": "2020-01-01",
        "end_date": "2024-01-01",
        "description": "Some description",
        "display_order": 0
    });

    let create_education_response = fixture
        .client()
        .post(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_education_json.to_string())
        .dispatch();

    assert_eq!(create_education_response.status(), Status::Created);
    let create_education_body = create_education_response
        .into_string()
        .expect("create education body");
    let create_education_json: Value =
        serde_json::from_str(&create_education_body).expect("valid json");
    let education_id = create_education_json["body"]["Education"]["id"]
        .as_i64()
        .expect("education id") as i32;

    let list_education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_education_response.status(), Status::Ok);
    let list_education_body = list_education_response
        .into_string()
        .expect("list education body");
    let list_education_json: Value =
        serde_json::from_str(&list_education_body).expect("valid json");
    let education_array = list_education_json["body"]["Educations"]
        .as_array()
        .expect("education array");
    assert!(
        education_array
            .iter()
            .any(|e| e["id"].as_i64() == Some(education_id as i64))
    );

    let update_education_payload = serde_json::json!({
        "degree": "BSc Computer Science (Honours)"
    });

    let update_education_response = fixture
        .client()
        .put(format!("/api/education/{}", education_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_education_payload.to_string())
        .dispatch();
    assert_eq!(update_education_response.status(), Status::Ok);

    let new_key_point_json = serde_json::json!({
        "key_point": "Graduated with honours",
        "display_order": 0
    });

    let create_key_point_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_key_point_json.to_string())
        .dispatch();
    assert_eq!(create_key_point_response.status(), Status::Created);
    let create_key_point_body = create_key_point_response
        .into_string()
        .expect("create key point body");
    let create_key_point_json: Value =
        serde_json::from_str(&create_key_point_body).expect("valid json");
    let key_point_id = create_key_point_json["body"]["EducationKeyPoint"]["id"]
        .as_i64()
        .expect("key point id") as i32;

    let list_key_points_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_key_points_response.status(), Status::Ok);

    let list_key_points_body = list_key_points_response
        .into_string()
        .expect("list education key points body");
    let list_key_points_json: Value =
        serde_json::from_str(&list_key_points_body).expect("valid json");
    let key_points_array = list_key_points_json["body"]["EducationKeyPoints"]
        .as_array()
        .expect("education key points array");
    assert!(
        key_points_array
            .iter()
            .any(|kp| kp["id"].as_i64() == Some(key_point_id as i64))
    );

    let update_key_point_payload = serde_json::json!({
        "key_point": "Graduated with first-class honours"
    });
    let update_key_point_response = fixture
        .client()
        .put(format!("/api/education_key_points/{}", key_point_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(update_key_point_payload.to_string())
        .dispatch();
    assert_eq!(update_key_point_response.status(), Status::Ok);

    let delete_key_point_response = fixture
        .client()
        .delete(format!("/api/education_key_points/{}", key_point_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_key_point_response.status(), Status::NoContent);

    let delete_education_response = fixture
        .client()
        .delete(format!("/api/education/{}", education_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_education_response.status(), Status::NoContent);
}

#[test]
fn test_work_experiences_and_key_points_crud_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "work.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Work User",
        "profile_image_url": null,
        "location": null,
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_resume_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_resume_response.status(), Status::Created);
    let create_resume_body = create_resume_response
        .into_string()
        .expect("create resume body");
    let create_resume_json: Value = serde_json::from_str(&create_resume_body).expect("valid json");
    let resume_id = create_resume_json["body"]["Resume"]["id"]
        .as_i64()
        .expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let new_work_json = serde_json::json!({
        "job_title": "Software Engineer",
        "company_name": "Example Corp",
        "start_date": "2022-01-01",
        "end_date": null,
        "description": "Did things",
        "display_order": 0
    });

    let create_work_response = fixture
        .client()
        .post(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_work_json.to_string())
        .dispatch();
    assert_eq!(create_work_response.status(), Status::Created);

    let create_work_body = create_work_response
        .into_string()
        .expect("create work body");
    let create_work_json: Value = serde_json::from_str(&create_work_body).expect("valid json");
    let work_id = create_work_json["body"]["WorkExperience"]["id"]
        .as_i64()
        .expect("work id") as i32;

    let list_work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_work_response.status(), Status::Ok);

    let list_work_body = list_work_response.into_string().expect("list work body");
    let list_work_json: Value = serde_json::from_str(&list_work_body).expect("valid json");
    let work_array = list_work_json["body"]["WorkExperiences"]
        .as_array()
        .expect("work experiences array");
    assert!(
        work_array
            .iter()
            .any(|w| w["id"].as_i64() == Some(work_id as i64))
    );

    let new_work_kp_json = serde_json::json!({
        "key_point": "Led a project",
        "display_order": 0
    });

    let create_work_kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            resume_id, work_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_work_kp_json.to_string())
        .dispatch();
    assert_eq!(create_work_kp_response.status(), Status::Created);

    let create_work_kp_body = create_work_kp_response
        .into_string()
        .expect("create work key point body");
    let create_work_kp_json: Value =
        serde_json::from_str(&create_work_kp_body).expect("valid json");
    let work_kp_id = create_work_kp_json["body"]["WorkExperienceKeyPoint"]["id"]
        .as_i64()
        .expect("work key point id") as i32;

    let delete_work_kp_response = fixture
        .client()
        .delete(format!("/api/work_experience_key_points/{}", work_kp_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_work_kp_response.status(), Status::NoContent);

    let list_work_kps_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            resume_id, work_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_work_kps_response.status(), Status::Ok);
    let list_work_kps_body = list_work_kps_response
        .into_string()
        .expect("list work key points body");
    let list_work_kps_json: Value = serde_json::from_str(&list_work_kps_body).expect("valid json");
    let work_kps_array = list_work_kps_json["body"]["WorkExperienceKeyPoints"]
        .as_array()
        .expect("work key points array");
    assert!(
        !work_kps_array
            .iter()
            .any(|kp| kp["id"].as_i64() == Some(work_kp_id as i64))
    );

    let delete_work_response = fixture
        .client()
        .delete(format!("/api/work_experiences/{}", work_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_work_response.status(), Status::NoContent);
}

#[test]
fn test_portfolio_projects_key_points_and_technologies_crud_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "portfolio.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Portfolio User",
        "profile_image_url": null,
        "location": null,
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_resume_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();

    assert_eq!(create_resume_response.status(), Status::Created);
    let create_resume_body = create_resume_response
        .into_string()
        .expect("create resume body");
    let create_resume_json: Value = serde_json::from_str(&create_resume_body).expect("valid json");
    let resume_id = create_resume_json["body"]["Resume"]["id"]
        .as_i64()
        .expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let new_project_json = serde_json::json!({
        "project_name": "Resume Builder",
        "image_url": null,
        "project_link": "https://example.com",
        "source_code_link": null,
        "description": "A project",
        "display_order": 0
    });

    let create_project_response = fixture
        .client()
        .post(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_project_json.to_string())
        .dispatch();
    assert_eq!(create_project_response.status(), Status::Created);

    let create_project_body = create_project_response
        .into_string()
        .expect("create project body");
    let create_project_json: Value =
        serde_json::from_str(&create_project_body).expect("valid json");
    let project_id = create_project_json["body"]["PortfolioProject"]["id"]
        .as_i64()
        .expect("project id") as i32;

    let list_projects_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_projects_response.status(), Status::Ok);
    let list_projects_body = list_projects_response
        .into_string()
        .expect("list projects body");
    let list_projects_json: Value = serde_json::from_str(&list_projects_body).expect("valid json");
    let projects_array = list_projects_json["body"]["PortfolioProjects"]
        .as_array()
        .expect("projects array");
    assert!(
        projects_array
            .iter()
            .any(|p| p["id"].as_i64() == Some(project_id as i64))
    );

    let new_project_kp_json = serde_json::json!({
        "key_point": "Built with Svelte",
        "display_order": 0
    });

    let create_project_kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/portfolio_projects/{}/key_points",
            resume_id, project_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_project_kp_json.to_string())
        .dispatch();
    assert_eq!(create_project_kp_response.status(), Status::Created);

    let create_project_kp_body = create_project_kp_response
        .into_string()
        .expect("create project key point body");
    let create_project_kp_json: Value =
        serde_json::from_str(&create_project_kp_body).expect("valid json");
    let project_kp_id = create_project_kp_json["body"]["PortfolioKeyPoint"]["id"]
        .as_i64()
        .expect("project key point id") as i32;

    let list_project_kps_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/portfolio_projects/{}/key_points",
            resume_id, project_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_project_kps_response.status(), Status::Ok);
    let list_project_kps_body = list_project_kps_response
        .into_string()
        .expect("list project key points body");
    let list_project_kps_json: Value =
        serde_json::from_str(&list_project_kps_body).expect("valid json");
    let project_kps_array = list_project_kps_json["body"]["PortfolioKeyPoints"]
        .as_array()
        .expect("project key points array");
    assert!(
        project_kps_array
            .iter()
            .any(|kp| kp["id"].as_i64() == Some(project_kp_id as i64))
    );

    let new_tech_json = serde_json::json!({
        "technology_name": "Rust",
        "display_order": 0
    });

    let create_tech_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/portfolio_projects/{}/technologies",
            resume_id, project_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_tech_json.to_string())
        .dispatch();
    assert_eq!(create_tech_response.status(), Status::Created);

    let create_tech_body = create_tech_response
        .into_string()
        .expect("create tech body");
    let create_tech_json: Value = serde_json::from_str(&create_tech_body).expect("valid json");
    let tech_id = create_tech_json["body"]["PortfolioTechnology"]["id"]
        .as_i64()
        .expect("tech id") as i32;

    let list_techs_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/portfolio_projects/{}/technologies",
            resume_id, project_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(list_techs_response.status(), Status::Ok);
    let list_techs_body = list_techs_response.into_string().expect("list techs body");
    let list_techs_json: Value = serde_json::from_str(&list_techs_body).expect("valid json");
    let techs_array = list_techs_json["body"]["PortfolioTechnologies"]
        .as_array()
        .expect("technologies array");
    assert!(
        techs_array
            .iter()
            .any(|t| t["id"].as_i64() == Some(tech_id as i64))
    );

    let delete_tech_response = fixture
        .client()
        .delete(format!("/api/portfolio_technologies/{}", tech_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_tech_response.status(), Status::NoContent);

    let delete_project_kp_response = fixture
        .client()
        .delete(format!("/api/portfolio_key_points/{}", project_kp_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_project_kp_response.status(), Status::NoContent);

    let delete_project_response = fixture
        .client()
        .delete(format!("/api/portfolio_projects/{}", project_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(delete_project_response.status(), Status::NoContent);
}
