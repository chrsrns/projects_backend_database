#![allow(dead_code)]
use api::realtime::Hub;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

static UNIQUE_EMAIL_COUNTER: AtomicU64 = AtomicU64::new(0);

pub mod proxy_mock;

pub fn unique_email(prefix: &str) -> String {
    let counter = UNIQUE_EMAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    format!("{}.{}.{}.{}@example.com", prefix, pid, nanos, counter)
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
    pub(crate) hub: Hub,
    created_resume_ids: Vec<i32>,
    created_user_ids: Vec<i32>,
    created_session_ids: Vec<String>,
    auth_token: String,
    lock_key: i64,
    lock_connection: PgConnection,
}

impl Fixture {
    pub fn new(lock_key: i64) -> Self {
        infrastructure::run_migrations_once().expect("run pending migrations");

        let mut lock_connection = establish_connection();
        diesel::sql_query(format!("SELECT pg_advisory_lock({})", lock_key))
            .execute(&mut lock_connection)
            .expect("acquire advisory lock");

        let hub = Hub::new();
        let rocket = api::build_rocket_with_hub(
            hub.clone(),
            shared::node_config::NodeConfig { port: 53421 },
        );
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let auth_ctx = register_and_login(&client, "realtime.tests");
        let auth_token = auth_ctx.token;
        let created_user_ids = vec![auth_ctx.user_id];
        let created_session_ids = vec![auth_token.clone()];

        Fixture {
            client,
            hub,
            created_resume_ids: Vec::new(),
            created_user_ids,
            created_session_ids,
            auth_token,
            lock_key,
            lock_connection,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    pub fn user_id(&self) -> i32 {
        self.created_user_ids[0]
    }

    pub fn auth_header(&self) -> Header<'static> {
        Header::new("Authorization", format!("Bearer {}", self.auth_token))
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

    pub fn untrack_resume_id(&mut self, id: i32) {
        self.created_resume_ids.retain(|&existing| existing != id);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let connection = &mut self.lock_connection;

        if !self.created_resume_ids.is_empty() {
            use domain::schema::resumes::dsl as resumes_dsl;

            // A variant references its base, so variants must be deleted first
            // or the base delete is rejected by the foreign key.
            let variant_ids: Vec<i32> = resumes_dsl::resumes
                .filter(resumes_dsl::id.eq_any(&self.created_resume_ids))
                .filter(resumes_dsl::base_resume_id.is_not_null())
                .select(resumes_dsl::id)
                .load(connection)
                .unwrap_or_default();

            let base_ids: Vec<i32> = self
                .created_resume_ids
                .iter()
                .copied()
                .filter(|id| !variant_ids.contains(id))
                .collect();

            for resume_id in variant_ids.iter().chain(base_ids.iter()) {
                match diesel::delete(resumes_dsl::resumes.find(resume_id)).execute(connection) {
                    Ok(_) => println!("✓ Deleted resume ID {}", resume_id),
                    Err(e) => eprintln!("✗ Failed to delete resume ID {}: {}", resume_id, e),
                }
            }
        }

        for session_id in &self.created_session_ids {
            let Ok(session_uuid) = session_id.parse::<uuid::Uuid>() else {
                continue;
            };
            let _ = diesel::sql_query("DELETE FROM sessions WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(session_uuid)
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
