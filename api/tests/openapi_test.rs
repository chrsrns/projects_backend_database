use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json::Value;

#[test]
fn test_openapi_json_is_served() {
    let rocket = api::build_rocket(shared::node_config::NodeConfig { port: 53421 });
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client.get("/api/openapi.json").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("openapi body");
    let json: Value = serde_json::from_str(&body).expect("valid openapi json");

    assert!(json.get("openapi").is_some());
    assert!(json.get("paths").is_some());
}
