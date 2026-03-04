use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;
use support::Fixture;

#[test]
fn test_register_login_me_logout_flow() {
    let mut fixture = Fixture::new(9_225_338);

    let unique_email = support::unique_email("auth.user");

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
        .header(support::auth_header(&token))
        .dispatch();

    assert_eq!(me_response.status(), Status::Ok);
    let me_body = me_response.into_string().expect("me body");
    let me_json: Value = serde_json::from_str(&me_body).expect("valid json");
    assert_eq!(me_json["body"]["id"].as_i64().unwrap() as i32, user_id);

    let logout_response = fixture
        .client()
        .post("/api/auth/logout")
        .header(support::auth_header(&token))
        .dispatch();
    assert_eq!(logout_response.status(), Status::Ok);

    let me_after_logout = fixture
        .client()
        .get("/api/auth/me")
        .header(support::auth_header(&token))
        .dispatch();

    assert_eq!(me_after_logout.status(), Status::Unauthorized);
}
