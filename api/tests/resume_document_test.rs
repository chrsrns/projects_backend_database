use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::Value;

fn client() -> Client {
    // Anonymous validate/convert endpoints take no DB, Hub, or auth state;
    // a bare Rocket build is the whole fixture.
    Client::tracked(api::build_rocket(shared::node_config::NodeConfig {
        port: 53981,
    }))
    .expect("valid rocket instance")
}

fn markdown_content_type() -> ContentType {
    ContentType::new("text", "markdown")
}

fn sample_markdown() -> String {
    "# Jane Doe\n\n- Location: New York, NY\n- Email: jane.doe@example.com\n- GitHub: https://github.com/janedoe\n- Mobile: +1987654321\n- Public: true\n\n## Summary\n\nExperienced backend developer.\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024)\n- Degree: Bachelor of Science\n- Description: Focused on software engineering\n- Graduated with honors\n- Specialized in distributed systems\n\n## Skills\n\n- Rust - 90%\n- Python - 75%\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (Jan 2020 - Present)\n- Description: Backend development\n- Led team of 5 developers\n- Improved system performance by 40%\n\n## Portfolio Projects\n\n### My Portfolio\n- Live: https://example.com\n- Source: https://github.com/janedoe/project\n- Technologies: Rust, Rocket, Diesel\n- Built a scalable resume API\n\n## Languages & Frameworks\n\n### Rust\n- Rocket\n- Actix\n\n### Python\n- Django\n"
    .to_string()
}

#[test]
fn test_validate_valid_markdown() {
    let client = client();
    let response = client
        .post("/api/resume/validate/markdown")
        .header(markdown_content_type())
        .body(sample_markdown())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let json: Value =
        serde_json::from_str(&response.into_string().expect("validate body")).expect("valid json");
    assert_eq!(json["body"]["valid"], true);
    assert_eq!(
        json["body"]["errors"]
            .as_array()
            .expect("errors array")
            .len(),
        0
    );
}

#[test]
fn test_validate_invalid_markdown_reports_error() {
    // Missing required Email field — parse fails but endpoint still returns
    // 200 with a report instead of an HTTP error.
    let markdown = "# Jane Doe\n\n## Skills\n\n- Rust - 90%\n";
    let client = client();
    let response = client
        .post("/api/resume/validate/markdown")
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let json: Value =
        serde_json::from_str(&response.into_string().expect("validate body")).expect("valid json");
    assert_eq!(json["body"]["valid"], false);
    let errors = json["body"]["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["section"], Value::Null);
    assert_eq!(errors[0]["message"], "Missing required field: Email");
}

#[test]
fn test_validate_oversized_markdown_is_413() {
    let big = "a".repeat(1_048_577);
    let client = client();
    let response = client
        .post("/api/resume/validate/markdown")
        .header(markdown_content_type())
        .body(big)
        .dispatch();

    assert_eq!(response.status(), Status::PayloadTooLarge);
}
