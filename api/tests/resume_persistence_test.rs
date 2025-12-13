#[macro_use]
extern crate rocket;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use std::time::Duration;

/// Test fixture that automatically cleans up created resumes
struct TestFixture {
    client: Client,
    created_resume_ids: Vec<i32>,
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

        let rocket = rocket::build().mount(
            "/api",
            routes![
                api::resume_handler::list_resumes_handler,
                api::resume_handler::list_resume_handler,
                api::resume_handler::create_resume_handler,
                api::resume_handler::update_resume_handler,
                api::resume_handler::delete_resume_handler,
            ],
        );

        let client = Client::tracked(rocket).expect("valid rocket instance");

        TestFixture {
            client,
            created_resume_ids: Vec::new(),
            lock_key,
            lock_connection,
        }
    }

    fn client(&self) -> &Client {
        &self.client
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
        "mobile_number": "+1234567890"
    });

    // Step 1: Create a new resume via POST
    let create_response = fixture
        .client()
        .post("/api/new_resume")
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
        "mobile_number": null
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
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
