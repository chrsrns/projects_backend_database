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

#[test]
fn test_convert_valid_markdown_returns_document() {
    let client = client();
    let response = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(sample_markdown())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let json: Value =
        serde_json::from_str(&response.into_string().expect("convert body")).expect("valid json");

    let envelope = &json["body"];
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["generator"], "projects_backend_database");

    let document = &envelope["document"];
    assert_eq!(document["resume"]["id"], 0);
    assert_eq!(document["resume"]["name"], "Jane Doe");
    assert_eq!(document["resume"]["is_public"], true);
    assert_eq!(
        document["resume"]["executive_summary"],
        "Experienced backend developer."
    );
    assert!(document["resume"].get("created_at").is_none());
    assert!(document["resume"].get("updated_at").is_none());
    assert!(document["resume"].get("created_by").is_none());

    let education = &document["education"];
    assert_eq!(education.as_array().expect("education array").len(), 1);
    assert_eq!(education[0]["id"], 1);
    assert_eq!(education[0]["resume_id"], 0);
    assert_eq!(education[0]["active"], true);
    assert_eq!(education[0]["start_date"], "2020-09");
    assert_eq!(education[0]["end_date"], "2024-05");
    assert_eq!(education[0]["degree"], "Bachelor of Science");
    assert_eq!(education[0]["institution_name"], "University of ABC");

    // Child maps are JSON objects keyed by the minted parent id as a string.
    let edu_kps = &document["education_key_points"];
    let first_parent = edu_kps["1"].as_array().expect("key points for parent 1");
    assert_eq!(first_parent.len(), 2);
    assert_eq!(first_parent[0]["education_id"], 1);
    assert_eq!(first_parent[0]["key_point"], "Graduated with honors");
    assert_eq!(first_parent[0]["active"], true);
    assert_eq!(first_parent[0]["id"], 1);
    assert_eq!(first_parent[1]["id"], 2);

    let skills = document["skills"].as_array().expect("skills array");
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0]["id"], 1);
    assert_eq!(skills[0]["resume_id"], 0);
    assert_eq!(skills[0]["skill_name"], "Rust");
    assert_eq!(skills[0]["confidence_percentage"], 90);
    assert!(skills[0].get("active").is_none());

    let work = &document["work_experiences"][0];
    assert_eq!(work["id"], 1);
    assert_eq!(work["job_title"], "Senior Software Engineer");
    assert_eq!(work["start_date"], "2020-01");
    assert_eq!(work["end_date"], Value::Null);
    let work_kps = &document["work_experience_key_points"]["1"];
    assert_eq!(work_kps.as_array().expect("work key points").len(), 2);

    let projects = &document["portfolio_projects"][0];
    assert_eq!(projects["id"], 1);
    assert_eq!(projects["project_name"], "My Portfolio");
    let techs = &document["portfolio_technologies"]["1"];
    assert_eq!(techs.as_array().expect("technologies").len(), 3);
    assert_eq!(techs[0]["technology_name"], "Rust");
    assert_eq!(techs[0]["portfolio_project_id"], 1);
    assert_eq!(techs[0]["id"], 1);
    assert_eq!(techs[1]["id"], 2);
    assert_eq!(techs[2]["id"], 3);

    let languages = document["languages"].as_array().expect("languages");
    assert_eq!(languages.len(), 2);
    assert_eq!(languages[0]["id"], 1);
    assert_eq!(languages[0]["language_name"], "Rust");
    let frameworks = &document["frameworks"]["1"];
    assert_eq!(frameworks.as_array().expect("frameworks").len(), 2);
    assert_eq!(frameworks[0]["language_id"], 1);
    assert_eq!(frameworks[0]["framework_name"], "Rocket");
    let second_frameworks = &document["frameworks"]["2"];
    assert_eq!(second_frameworks[0]["language_id"], 2);
    assert_eq!(second_frameworks[0]["framework_name"], "Django");
    assert_eq!(second_frameworks[0]["id"], 3);
}

#[test]
fn test_convert_invalid_markdown_is_400() {
    let client = client();
    let markdown = "# Jane Doe\n\n## Skills\n\n- Rust - 90%\n";
    let response = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    let json: Value =
        serde_json::from_str(&response.into_string().expect("error body")).expect("valid json");
    assert_eq!(json["body"], "Missing required field: Email");
}

#[test]
fn test_convert_oversized_markdown_is_413() {
    let client = client();
    let big = "a".repeat(1_048_577);
    let response = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(big)
        .dispatch();

    assert_eq!(response.status(), Status::PayloadTooLarge);
}

#[test]
fn test_document_schema_endpoint() {
    let client = client();
    let response = client.get("/api/resume/document-schema").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.content_type().expect("content type"),
        ContentType::JSON
    );
    let json: Value =
        serde_json::from_str(&response.into_string().expect("schema body")).expect("valid json");

    let schemas = &json["components"]["schemas"];
    let resume_document = schemas
        .get("ResumeDocument")
        .expect("ResumeDocument schema present");
    let properties = resume_document["properties"]
        .as_object()
        .expect("ResumeDocument properties");
    for field in [
        "resume",
        "education",
        "education_key_points",
        "skills",
        "work_experiences",
        "work_experience_key_points",
        "portfolio_projects",
        "portfolio_key_points",
        "portfolio_technologies",
        "languages",
        "frameworks",
    ] {
        assert!(
            properties.contains_key(field),
            "ResumeDocument property {} missing",
            field
        );
    }

    // Every $ref inside the emitted schemas resolves within the same
    // components.schemas map — the response is self-contained.
    let schema_map = schemas.as_object().expect("schemas object");
    fn collect_refs(value: &Value, refs: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, v) in map {
                    if key == "$ref" {
                        if let Some(r) = v.as_str() {
                            refs.push(r.to_string());
                        }
                    } else {
                        collect_refs(v, refs);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| collect_refs(v, refs)),
            _ => {}
        }
    }
    let mut refs = Vec::new();
    collect_refs(schemas, &mut refs);
    for r in &refs {
        let name = r
            .strip_prefix("#/components/schemas/")
            .expect("component-local ref");
        assert!(
            schema_map.contains_key(name),
            "unresolved $ref {} in document-schema",
            r
        );
    }
    assert!(!refs.is_empty(), "expected nested $refs in schema");
}

#[test]
fn test_convert_is_deterministic() {
    let client = client();
    let markdown = sample_markdown();

    let first = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(markdown.clone())
        .dispatch()
        .into_string()
        .expect("first body");
    let second = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(markdown)
        .dispatch()
        .into_string()
        .expect("second body");

    assert_eq!(first, second, "convert output must be byte-identical");
}

#[test]
fn test_convert_date_encoding() {
    let client = client();
    // Year-only start, month-year end, and `Present` end date.
    let markdown = "# Jane Doe\n\n- Email: jane.doe@example.com\n\n## Education\n\n### Bachelor - Uni (2019 - May 2023)\n\n## Work Experience\n\n### Dev - Corp (2020 - Present)\n";
    let response = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let json: Value =
        serde_json::from_str(&response.into_string().expect("convert body")).expect("valid json");
    let document = &json["body"]["document"];

    assert_eq!(document["education"][0]["start_date"], "2019");
    assert_eq!(document["education"][0]["end_date"], "2023-05");
    assert_eq!(document["work_experiences"][0]["start_date"], "2020");
    assert_eq!(document["work_experiences"][0]["end_date"], Value::Null);
}

#[test]
fn test_convert_child_maps_across_multiple_parents() {
    let client = client();
    let markdown = "# Jane Doe\n\n- Email: jane.doe@example.com\n\n## Education\n\n### Bachelor - Uni A (2016 - 2020)\n- first edu kp\n\n### Master - Uni B (2020 - 2022)\n- second edu kp\n- another kp\n\n## Languages & Frameworks\n\n### Rust\n- Rocket\n\n### Go\n- Gin\n- Echo\n";
    let response = client
        .post("/api/resume/convert/markdown")
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let json: Value =
        serde_json::from_str(&response.into_string().expect("convert body")).expect("valid json");
    let document = &json["body"]["document"];

    let edu_kps = document["education_key_points"].as_object().expect("map");
    // Ascending numeric key order via BTreeMap.
    let keys: Vec<&String> = edu_kps.keys().collect();
    assert_eq!(keys, ["1", "2"]);
    assert_eq!(edu_kps["1"][0]["key_point"], "first edu kp");
    assert_eq!(edu_kps["1"][0]["education_id"], 1);
    assert_eq!(edu_kps["2"][0]["key_point"], "second edu kp");
    assert_eq!(edu_kps["2"][0]["education_id"], 2);
    // Global 1-based child ids across the whole map in document order.
    assert_eq!(edu_kps["1"][0]["id"], 1);
    assert_eq!(edu_kps["2"][0]["id"], 2);
    assert_eq!(edu_kps["2"][1]["id"], 3);

    let frameworks = document["frameworks"].as_object().expect("framework map");
    let fw_keys: Vec<&String> = frameworks.keys().collect();
    assert_eq!(fw_keys, ["1", "2"]);
    assert_eq!(frameworks["2"][0]["language_id"], 2);
    assert_eq!(frameworks["2"][1]["language_id"], 2);
    assert_eq!(frameworks["1"][0]["id"], 1);
    assert_eq!(frameworks["2"][0]["id"], 2);
    assert_eq!(frameworks["2"][1]["id"], 3);
}

#[test]
fn test_handlers_take_only_markdown_argument() {
    // Direct calls compile only because the handler signatures take a single
    // LimitedMarkdown — no database, Hub, or auth session parameters.
    use api::route_handlers::resume::markdown_handler::{
        LimitedMarkdown, convert_resume_markdown, validate_resume_markdown,
    };

    let report = validate_resume_markdown(LimitedMarkdown(sample_markdown()));
    assert!(report.0.body.valid);
    assert!(report.0.body.errors.is_empty());

    let invalid = validate_resume_markdown(LimitedMarkdown("## Nope".to_string()));
    assert!(!invalid.0.body.valid);
    assert_eq!(invalid.0.body.errors.len(), 1);
    assert_eq!(invalid.0.body.errors[0].section, None);

    let envelope = match convert_resume_markdown(LimitedMarkdown(sample_markdown())) {
        Ok(json) => json.0.body,
        Err(_) => panic!("convert should succeed"),
    };
    assert_eq!(envelope.schema_version, 1);
    assert_eq!(envelope.generator, "projects_backend_database");
    assert_eq!(envelope.document.resume.id, 0);
    assert_eq!(envelope.document.education[0].id, 1);
    assert_eq!(
        envelope.document.education_key_points[&1][0].education_id,
        1
    );

    match convert_resume_markdown(LimitedMarkdown("## Nope".to_string())) {
        Ok(_) => panic!("convert should fail"),
        Err(err) => {
            assert_eq!(err.0, Status::BadRequest);
            assert_eq!(
                err.1.0.body,
                "Unknown section 'Nope'. Expected one of: Education, Skills, Work Experience, Portfolio Projects, Languages & Frameworks, Summary"
            );
        }
    }
}
