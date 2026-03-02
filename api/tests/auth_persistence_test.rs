use diesel::pg::PgConnection;
use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;

/// Test fixture that automatically cleans up created users/sessions
struct TestFixture {
    client: Client,
    created_user_ids: Vec<i32>,
    created_session_ids: Vec<String>,
    lock_key: i64,
    lock_connection: PgConnection,
}

impl TestFixture {
    fn new() -> Self {
        let lock_key: i64 = 9_225_338;
        let mut lock_connection = establish_connection();
        diesel::sql_query(format!("SELECT pg_advisory_lock({})", lock_key))
            .execute(&mut lock_connection)
            .expect("acquire advisory lock");

        let rocket = api::build_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        TestFixture {
            client,
            created_user_ids: Vec::new(),
            created_session_ids: Vec::new(),
            lock_key,
            lock_connection,
        }
    }

    fn client(&self) -> &Client {
        &self.client
    }

    fn track_user_id(&mut self, id: i32) {
        self.created_user_ids.push(id);
    }

    fn track_session_id(&mut self, id: String) {
        self.created_session_ids.push(id);
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        use domain::schema::users::dsl as users_dsl;

        let connection = &mut self.lock_connection;

        for session_id in &self.created_session_ids {
            let _ = diesel::sql_query("DELETE FROM sessions WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(
                    session_id
                        .parse::<uuid::Uuid>()
                        .expect("valid uuid session id"),
                )
                .execute(connection);
        }

        for user_id in &self.created_user_ids {
            let _ = diesel::delete(users_dsl::users.find(user_id)).execute(connection);
        }

        diesel::sql_query(format!("SELECT pg_advisory_unlock({})", self.lock_key))
            .execute(&mut self.lock_connection)
            .expect("release advisory lock");
    }
}

#[test]
fn test_register_login_me_logout_flow() {
    let mut fixture = TestFixture::new();

    let unique_email = format!(
        "auth.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let register_payload = serde_json::json!({
        "email": unique_email,
        "password": "test-password"
    });

    let register_response = fixture
        .client()
        .post("/api/auth/register")
        .header(ContentType::JSON)
        .body(register_payload.to_string())
        .dispatch();

    assert_eq!(register_response.status(), Status::Created);

    let register_body = register_response.into_string().expect("register body");
    let register_json: Value = serde_json::from_str(&register_body).expect("valid json");
    let created_user = &register_json["body"];
    let user_id = created_user["id"].as_i64().expect("user id") as i32;
    fixture.track_user_id(user_id);

    let login_payload = serde_json::json!({
        "email": created_user["email"].as_str().expect("email"),
        "password": "test-password"
    });

    let login_response = fixture
        .client()
        .post("/api/auth/login")
        .header(ContentType::JSON)
        .body(login_payload.to_string())
        .dispatch();

    assert_eq!(login_response.status(), Status::Ok);
    let login_body = login_response.into_string().expect("login body");
    let login_json: Value = serde_json::from_str(&login_body).expect("valid json");

    let token = login_json["body"]["token"]
        .as_str()
        .expect("token")
        .to_string();

    fixture.track_session_id(token.clone());

    let me_response = fixture
        .client()
        .get("/api/auth/me")
        .header(Header::new("Authorization", format!("Bearer {}", token)))
        .dispatch();

    assert_eq!(me_response.status(), Status::Ok);
    let me_body = me_response.into_string().expect("me body");
    let me_json: Value = serde_json::from_str(&me_body).expect("valid json");
    assert_eq!(me_json["body"]["id"].as_i64().unwrap() as i32, user_id);

    let logout_response = fixture
        .client()
        .post("/api/auth/logout")
        .header(Header::new("Authorization", format!("Bearer {}", token)))
        .dispatch();
    assert_eq!(logout_response.status(), Status::Ok);

    let me_after_logout = fixture
        .client()
        .get("/api/auth/me")
        .header(Header::new("Authorization", format!("Bearer {}", token)))
        .dispatch();

    assert_eq!(me_after_logout.status(), Status::Unauthorized);
}
