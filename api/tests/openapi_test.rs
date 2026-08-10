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

    let import_responses = json
        .get("paths")
        .and_then(|p| p.get("/resume/import/markdown"))
        .and_then(|m| m.get("post"))
        .and_then(|op| op.get("responses"))
        .expect("import markdown responses");
    assert!(
        import_responses.get("413").is_some(),
        "expected 413 response documented for markdown import"
    );

    let schemas = json
        .get("components")
        .and_then(|c| c.get("schemas"))
        .expect("schemas");

    for schema_name in ["Resume", "NewResumeRequest", "UpdateResume"] {
        let video = schemas
            .get(schema_name)
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.get("video"))
            .expect(&format!("video property in {}", schema_name));

        let types = video
            .get("type")
            .and_then(|t| t.as_array())
            .expect("video type array");
        assert!(types.iter().any(|t| t == "string"), "video is a string");

        let max_length = video
            .get("maxLength")
            .and_then(|m| m.as_u64())
            .expect("video maxLength");
        assert_eq!(max_length, 500, "video maxLength is 500");
    }

    for schema_name in [
        "PortfolioProject",
        "NewPortfolioProjectRequest",
        "UpdatePortfolioProject",
    ] {
        let video_url = schemas
            .get(schema_name)
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.get("video_url"))
            .expect(&format!("video_url property in {}", schema_name));

        let types = video_url
            .get("type")
            .and_then(|t| t.as_array())
            .expect("video_url type array");
        assert!(types.iter().any(|t| t == "string"), "video_url is a string");

        let max_length = video_url
            .get("maxLength")
            .and_then(|m| m.as_u64())
            .expect("video_url maxLength");
        assert_eq!(max_length, 500, "video_url maxLength is 500");

        for field in ["image_url", "project_link", "source_code_link"] {
            let property = schemas
                .get(schema_name)
                .and_then(|s| s.get("properties"))
                .and_then(|p| p.get(field))
                .expect(&format!("{} property in {}", field, schema_name));

            let types = property
                .get("type")
                .and_then(|t| t.as_array())
                .expect(&format!("{} type array in {}", field, schema_name));
            assert!(
                types.iter().any(|t| t == "string"),
                "{} is a string in {}",
                field,
                schema_name
            );

            let max_length = property
                .get("maxLength")
                .and_then(|m| m.as_u64())
                .expect(&format!("{} maxLength in {}", field, schema_name));
            assert_eq!(
                max_length, 500,
                "{} maxLength is 500 in {}",
                field, schema_name
            );
        }
    }
}
