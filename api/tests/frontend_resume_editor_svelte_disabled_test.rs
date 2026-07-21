mod support;

#[cfg(not(feature = "frontend_resume_editor_svelte"))]
#[test]
fn test_feature_disabled_returns_404() {
    let rocket = api::build_rocket_with_hub(
        api::realtime::Hub::new(),
        shared::node_config::NodeConfig { port: 53421 },
    );
    let client = rocket::local::blocking::Client::tracked(rocket).expect("valid rocket");

    let response = client.get("/resume_editor/foo").dispatch();

    assert_eq!(response.status(), rocket::http::Status::NotFound);
}
