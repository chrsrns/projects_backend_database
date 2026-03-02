use diesel::pg::PgConnection;
use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;

use api::realtime::{Hub, ResumeChangedAction};

/// Test fixture that automatically cleans up created resumes
struct TestFixture {
    client: Client,
    hub: Hub,
    created_resume_ids: Vec<i32>,
    auth_token: String,
    lock_key: i64,
    lock_connection: PgConnection,
}

impl TestFixture {
    fn new() -> Self {
        let lock_key: i64 = 9_225_339;
        let mut lock_connection = establish_connection();
        diesel::sql_query(format!("SELECT pg_advisory_lock({})", lock_key))
            .execute(&mut lock_connection)
            .expect("acquire advisory lock");

        let hub = Hub::new();
        let rocket = api::build_rocket_with_hub(hub.clone());
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let auth_token = {
            let unique_email = format!(
                "realtime.tests.{}@example.com",
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
            login_json["body"]["token"]
                .as_str()
                .expect("token")
                .to_string()
        };

        TestFixture {
            client,
            hub,
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
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        if !self.created_resume_ids.is_empty() {
            use domain::schema::resumes::dsl::*;
            let connection = &mut self.lock_connection;

            for resume_id_value in &self.created_resume_ids {
                let _ = diesel::delete(resumes.find(resume_id_value)).execute(connection);
            }
        }

        diesel::sql_query(format!("SELECT pg_advisory_unlock({})", self.lock_key))
            .execute(&mut self.lock_connection)
            .expect("release advisory lock");
    }
}

#[test]
fn test_resume_update_publishes_resume_changed_event() {
    let mut fixture = TestFixture::new();

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
    assert_eq!(evt.action, ResumeChangedAction::Updated);
}
