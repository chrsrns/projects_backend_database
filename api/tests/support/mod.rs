#![allow(dead_code)]
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use infrastructure::establish_connection;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use std::sync::OnceLock;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../infrastructure/migrations");

static MIGRATIONS_RAN: OnceLock<()> = OnceLock::new();

pub fn run_migrations_once() {
    MIGRATIONS_RAN.get_or_init(|| {
        let lock_key: i64 = 9_225_300;
        let mut connection = establish_connection();

        diesel::sql_query(format!("SELECT pg_advisory_lock({})", lock_key))
            .execute(&mut connection)
            .expect("acquire migrations advisory lock");

        connection
            .run_pending_migrations(MIGRATIONS)
            .expect("run pending migrations");

        diesel::sql_query(format!("SELECT pg_advisory_unlock({})", lock_key))
            .execute(&mut connection)
            .expect("release migrations advisory lock");
    });
}

pub fn unique_email(prefix: &str) -> String {
    format!(
        "{}.{}@example.com",
        prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

pub fn auth_header(token: &str) -> Header<'static> {
    Header::new("Authorization", format!("Bearer {}", token))
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i32,
    pub token: String,
}

pub fn register_and_login(client: &Client, email_prefix: &str) -> AuthContext {
    let email = unique_email(email_prefix);

    let register_payload = serde_json::json!({
        "email": email,
        "password": "test-password"
    });

    let register_response = client
        .post("/api/auth/register")
        .header(ContentType::JSON)
        .body(register_payload.to_string())
        .dispatch();

    assert_eq!(
        register_response.status(),
        Status::Created,
        "register should succeed"
    );

    let register_body = register_response.into_string().expect("register body");
    let register_json: Value = serde_json::from_str(&register_body).expect("valid json");
    let user_id = register_json["body"]["id"].as_i64().expect("user id") as i32;

    let login_payload = serde_json::json!({
        "email": register_json["body"]["email"].as_str().expect("email"),
        "password": "test-password"
    });

    let login_response = client
        .post("/api/auth/login")
        .header(ContentType::JSON)
        .body(login_payload.to_string())
        .dispatch();

    assert_eq!(login_response.status(), Status::Ok, "login should succeed");

    let login_body = login_response.into_string().expect("login body");
    let login_json: Value = serde_json::from_str(&login_body).expect("valid json");

    let token = login_json["body"]["token"]
        .as_str()
        .expect("token")
        .to_string();

    AuthContext { user_id, token }
}

pub struct Fixture {
    client: Client,
    created_resume_ids: Vec<i32>,
    created_user_ids: Vec<i32>,
    created_session_ids: Vec<String>,
    lock_key: i64,
    lock_connection: PgConnection,
}

impl Fixture {
    pub fn new(lock_key: i64) -> Self {
        run_migrations_once();

        let mut lock_connection = establish_connection();
        diesel::sql_query(format!("SELECT pg_advisory_lock({})", lock_key))
            .execute(&mut lock_connection)
            .expect("acquire advisory lock");

        let rocket = api::build_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        Fixture {
            client,
            created_resume_ids: Vec::new(),
            created_user_ids: Vec::new(),
            created_session_ids: Vec::new(),
            lock_key,
            lock_connection,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn track_resume_id(&mut self, id: i32) {
        self.created_resume_ids.push(id);
    }

    pub fn track_user_id(&mut self, id: i32) {
        self.created_user_ids.push(id);
    }

    pub fn track_session_id(&mut self, id: String) {
        self.created_session_ids.push(id);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let connection = &mut self.lock_connection;

        if !self.created_resume_ids.is_empty() {
            use domain::schema::resumes::dsl::*;

            for resume_id_value in &self.created_resume_ids {
                let _ = diesel::delete(resumes.find(resume_id_value)).execute(connection);
            }
        }

        for session_id in &self.created_session_ids {
            let _ = diesel::sql_query("DELETE FROM sessions WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(
                    session_id
                        .parse::<uuid::Uuid>()
                        .expect("valid uuid session id"),
                )
                .execute(connection);
        }

        if !self.created_user_ids.is_empty() {
            use domain::schema::users::dsl as users_dsl;

            for user_id in &self.created_user_ids {
                let _ = diesel::delete(users_dsl::users.find(user_id)).execute(connection);
            }
        }

        diesel::sql_query(format!("SELECT pg_advisory_unlock({})", self.lock_key))
            .execute(&mut self.lock_connection)
            .expect("release advisory lock");
    }
}
