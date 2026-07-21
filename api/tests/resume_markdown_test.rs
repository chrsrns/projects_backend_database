use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn markdown_content_type() -> ContentType {
    ContentType::new("text", "markdown")
}

fn unique_markdown_email(lock_key: i64) -> String {
    format!(
        "markdown.test.{}.{}@example.com",
        lock_key,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

fn sample_markdown() -> String {
    "# Jane Doe\n\n- Location: New York, NY\n- Email: jane.doe@example.com\n- GitHub: https://github.com/janedoe\n- Mobile: +1987654321\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024)\n- Degree: Bachelor of Science\n- Description: Focused on software engineering\n- Graduated with honors\n- Specialized in distributed systems\n\n## Skills\n\n- Rust - 90%\n- Python - 75%\n- JavaScript - 60%\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (Jan 2020 - Present)\n- Description: Backend development\n- Led team of 5 developers\n- Improved system performance by 40%\n\n## Portfolio Projects\n\n### My Portfolio\n- Live: https://example.com\n- Source: https://github.com/janedoe/project\n- Technologies: Rust, Rocket, Diesel\n- Built a scalable resume API\n\n## Languages & Frameworks\n\n### Rust\n- Rocket\n- Actix\n\n### Python\n- Django\n- FastAPI\n"
    .to_string()
}

fn create_full_resume(fixture: &mut support::Fixture, lock_key: i64) -> (i32, String) {
    let unique_email = format!(
        "markdown.user.{}.{}@example.com",
        lock_key,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "John Doe",
        "profile_image_url": "https://example.com/profile.jpg",
        "location": "San Francisco, CA",
        "email": unique_email,
        "github_url": "https://github.com/johndoe",
        "mobile_number": "+1234567890",
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

    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let resume_id = create_json["body"]["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;
    fixture.track_resume_id(resume_id);

    let education_response = fixture
        .client()
        .post(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "education_stage": "Bachelor's in Computer Science",
                "institution_name": "University of XYZ",
                "degree": "Bachelor of Science",
                "start_date": "2020-09-01",
                "end_date": "2024-05-01",
                "description": "Focused on software engineering",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(education_response.status(), Status::Created);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_id = education_json["body"]["id"].as_i64().expect("education id") as i32;

    let kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, education_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "key_point": "Graduated with honors",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(kp_response.status(), Status::Created);

    let skill_response = fixture
        .client()
        .post(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "skill_name": "Rust",
                "confidence_percentage": 90,
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(skill_response.status(), Status::Created);

    let work_response = fixture
        .client()
        .post(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "job_title": "Senior Software Engineer",
                "company_name": "Tech Corp",
                "start_date": "2020-01-01",
                "end_date": null,
                "description": "Backend development",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(work_response.status(), Status::Created);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_id = work_json["body"]["id"].as_i64().expect("work id") as i32;

    let work_kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            resume_id, work_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "key_point": "Led team of 5 developers",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(work_kp_response.status(), Status::Created);

    let project_response = fixture
        .client()
        .post(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "project_name": "My Portfolio",
                "image_url": "https://example.com/image.png",
                "project_link": "https://example.com",
                "source_code_link": "https://github.com/johndoe/project",
                "description": "Built a scalable resume API",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(project_response.status(), Status::Created);
    let project_body = project_response.into_string().expect("project body");
    let project_json: Value = serde_json::from_str(&project_body).expect("valid json");
    let project_id = project_json["body"]["id"].as_i64().expect("project id") as i32;

    let project_kp_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/portfolio_projects/{}/key_points",
            resume_id, project_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "key_point": "Implemented real-time updates",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(project_kp_response.status(), Status::Created);

    let project_tech_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/portfolio_projects/{}/technologies",
            resume_id, project_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "technology_name": "Rust",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(project_tech_response.status(), Status::Created);

    let language_response = fixture
        .client()
        .post(format!("/api/resume/{}/languages", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "language_name": "Rust",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(language_response.status(), Status::Created);
    let language_body = language_response.into_string().expect("language body");
    let language_json: Value = serde_json::from_str(&language_body).expect("valid json");
    let language_id = language_json["body"]["id"].as_i64().expect("language id") as i32;

    let framework_response = fixture
        .client()
        .post(format!(
            "/api/resume/{}/languages/{}/frameworks",
            resume_id, language_id
        ))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "framework_name": "Rocket",
                "display_order": 1
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(framework_response.status(), Status::Created);

    (resume_id, unique_email)
}

#[test]
fn test_export_resume_markdown() {
    let mut fixture = support::Fixture::new(9_227_001);
    let (resume_id, _email) = create_full_resume(&mut fixture, 9_227_001);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(export_response.status(), Status::Ok);
    let markdown = export_response.into_string().expect("markdown body");

    assert!(markdown.contains("# John Doe"));
    assert!(markdown.contains("## Education"));
    assert!(markdown.contains("## Skills"));
    assert!(markdown.contains("## Work Experience"));
    assert!(markdown.contains("## Portfolio Projects"));
    assert!(markdown.contains("## Languages & Frameworks"));
    assert!(markdown.contains("- Rust - 90%"));
    assert!(markdown.contains("- Led team of 5 developers"));
    assert!(markdown.contains("- Technologies: Rust"));
    assert!(markdown.contains("- Rocket"));
    assert!(markdown.contains("- GitHub: https://github.com/johndoe"));
    assert!(markdown.contains("- Profile Image: https://example.com/profile.jpg"));
    assert!(markdown.contains("- Live: https://example.com"));
    assert!(markdown.contains("- Source: https://github.com/johndoe/project"));
}

#[test]
fn test_import_resume_markdown() {
    let mut fixture = support::Fixture::new(9_227_002);

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(sample_markdown())
        .dispatch();

    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    assert_eq!(import_json["body"]["name"], "Jane Doe");
    assert_eq!(import_json["body"]["email"], "jane.doe@example.com");
    assert_eq!(import_json["body"]["location"], "New York, NY");

    let get_response = fixture
        .client()
        .get(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(get_response.status(), Status::Ok);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["institution_name"], "University of ABC");
    assert_eq!(education_items[0]["degree"], "Bachelor of Science");
    assert_eq!(education_items[0]["start_date"], "2020-09");
    assert_eq!(education_items[0]["end_date"], "2024-05");
    assert!(education_items[0].get("start_date_precision").is_none());
    assert!(education_items[0].get("end_date_precision").is_none());

    let skills_response = fixture
        .client()
        .get(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(skills_response.status(), Status::Ok);
    let skills_body = skills_response.into_string().expect("skills body");
    let skills_json: Value = serde_json::from_str(&skills_body).expect("valid json");
    let skills = skills_json["body"].as_array().expect("array");
    assert_eq!(skills.len(), 3);
    assert!(
        skills
            .iter()
            .any(|s| s["skill_name"] == "Rust" && s["confidence_percentage"] == 90)
    );

    let work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["job_title"], "Senior Software Engineer");
    assert_eq!(work_items[0]["company_name"], "Tech Corp");
    assert_eq!(work_items[0]["start_date"], "2020-01");
    assert!(work_items[0]["end_date"].is_null());
    assert!(work_items[0].get("start_date_precision").is_none());
    assert!(work_items[0].get("end_date_precision").is_none());

    let projects_response = fixture
        .client()
        .get(format!("/api/resume/{}/portfolio_projects", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(projects_response.status(), Status::Ok);
    let projects_body = projects_response.into_string().expect("projects body");
    let projects_json: Value = serde_json::from_str(&projects_body).expect("valid json");
    let projects = projects_json["body"].as_array().expect("array");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["project_name"], "My Portfolio");
    assert_eq!(projects[0]["project_link"], "https://example.com");
    assert_eq!(
        projects[0]["source_code_link"],
        "https://github.com/janedoe/project"
    );

    let languages_response = fixture
        .client()
        .get(format!("/api/resume/{}/languages", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(languages_response.status(), Status::Ok);
    let languages_body = languages_response.into_string().expect("languages body");
    let languages_json: Value = serde_json::from_str(&languages_body).expect("valid json");
    let languages = languages_json["body"].as_array().expect("array");
    assert_eq!(languages.len(), 2);
    assert!(languages.iter().any(|l| l["language_name"] == "Rust"));
}

#[test]
fn test_import_resume_markdown_with_iso_dates() {
    let mut fixture = support::Fixture::new(9_227_018);
    let unique_email = unique_markdown_email(9_227_018);
    let markdown = format!(
        "# Jane Doe\n\n- Email: {}\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (2020-09-01 - 2024-05-01)\n- Degree: Bachelor of Science\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (2020-01-15 - 2023-08-30)\n- Description: Backend development\n",
        unique_email
    );

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["start_date"], "2020-09-01");
    assert_eq!(education_items[0]["end_date"], "2024-05-01");
    assert!(education_items[0].get("start_date_precision").is_none());
    assert!(education_items[0].get("end_date_precision").is_none());

    let work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["start_date"], "2020-01-15");
    assert_eq!(work_items[0]["end_date"], "2023-08-30");
    assert!(work_items[0].get("start_date_precision").is_none());
    assert!(work_items[0].get("end_date_precision").is_none());
}

#[test]
fn test_import_resume_markdown_with_full_month_names() {
    let mut fixture = support::Fixture::new(9_227_019);
    let unique_email = unique_markdown_email(9_227_019);
    let markdown = format!(
        "# Jane Doe\n\n- Email: {}\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (September 2020 - May 2024)\n- Degree: Bachelor of Science\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (January 1919 - Present)\n- Description: Backend development\n",
        unique_email
    );

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["institution_name"], "University of ABC");
    assert_eq!(education_items[0]["degree"], "Bachelor of Science");
    assert_eq!(education_items[0]["start_date"], "2020-09");
    assert_eq!(education_items[0]["end_date"], "2024-05");
    assert!(education_items[0].get("start_date_precision").is_none());
    assert!(education_items[0].get("end_date_precision").is_none());

    let work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["job_title"], "Senior Software Engineer");
    assert_eq!(work_items[0]["company_name"], "Tech Corp");
    assert_eq!(work_items[0]["start_date"], "1919-01");
    assert!(work_items[0]["end_date"].is_null());
    assert!(work_items[0].get("start_date_precision").is_none());
    assert!(work_items[0].get("end_date_precision").is_none());
}

#[test]
fn test_export_resume_markdown_round_trip() {
    let mut fixture = support::Fixture::new(9_227_003);
    let (resume_id, original_email) = create_full_resume(&mut fixture, 9_227_003);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export_response.status(), Status::Ok);
    let markdown = export_response.into_string().expect("markdown body");

    let unique_import_email = format!(
        "roundtrip.user.{}.{}@example.com",
        9_227_003,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let markdown_for_import = markdown.replace(
        &format!("- Email: {}", original_email),
        &format!("- Email: {}", unique_import_email),
    );

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown_for_import)
        .dispatch();
    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let imported_resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(imported_resume_id);

    assert_eq!(import_json["body"]["name"], "John Doe");
    assert_eq!(import_json["body"]["email"], unique_import_email);
    assert_eq!(import_json["body"]["location"], "San Francisco, CA");
    assert_eq!(
        import_json["body"]["profile_image_url"],
        "https://example.com/profile.jpg"
    );
    assert_eq!(
        import_json["body"]["github_url"],
        "https://github.com/johndoe"
    );
    assert_eq!(import_json["body"]["mobile_number"], "+1234567890");
    assert_eq!(import_json["body"]["is_public"], true);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", imported_resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["institution_name"], "University of XYZ");
    assert_eq!(
        education_items[0]["education_stage"],
        "Bachelor's in Computer Science"
    );
    assert_eq!(education_items[0]["degree"], "Bachelor of Science");
    assert_eq!(education_items[0]["start_date"], "2020-09-01");
    assert_eq!(education_items[0]["end_date"], "2024-05-01");

    let skills_response = fixture
        .client()
        .get(format!("/api/resume/{}/skills", imported_resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(skills_response.status(), Status::Ok);
    let skills_body = skills_response.into_string().expect("skills body");
    let skills_json: Value = serde_json::from_str(&skills_body).expect("valid json");
    let skills = skills_json["body"].as_array().expect("array");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["skill_name"], "Rust");
    assert_eq!(skills[0]["confidence_percentage"], 90);

    let work_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/work_experiences",
            imported_resume_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["job_title"], "Senior Software Engineer");
    assert_eq!(work_items[0]["company_name"], "Tech Corp");
    assert_eq!(work_items[0]["description"], "Backend development");
    assert_eq!(work_items[0]["start_date"], "2020-01-01");
    assert!(work_items[0]["end_date"].is_null());

    let work_kp_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/work_experiences/{}/key_points",
            imported_resume_id,
            work_items[0]["id"].as_i64().expect("work id") as i32
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_kp_response.status(), Status::Ok);
    let work_kp_body = work_kp_response.into_string().expect("work kp body");
    let work_kp_json: Value = serde_json::from_str(&work_kp_body).expect("valid json");
    let work_kps = work_kp_json["body"].as_array().expect("array");
    assert_eq!(work_kps.len(), 1);
    assert_eq!(work_kps[0]["key_point"], "Led team of 5 developers");

    let projects_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/portfolio_projects",
            imported_resume_id
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(projects_response.status(), Status::Ok);
    let projects_body = projects_response.into_string().expect("projects body");
    let projects_json: Value = serde_json::from_str(&projects_body).expect("valid json");
    let projects = projects_json["body"].as_array().expect("array");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["project_name"], "My Portfolio");
    assert_eq!(projects[0]["project_link"], "https://example.com");
    assert_eq!(
        projects[0]["source_code_link"],
        "https://github.com/johndoe/project"
    );
    assert_eq!(projects[0]["image_url"], "https://example.com/image.png");
    assert_eq!(projects[0]["description"], "Built a scalable resume API");

    let languages_response = fixture
        .client()
        .get(format!("/api/resume/{}/languages", imported_resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(languages_response.status(), Status::Ok);
    let languages_body = languages_response.into_string().expect("languages body");
    let languages_json: Value = serde_json::from_str(&languages_body).expect("valid json");
    let languages = languages_json["body"].as_array().expect("array");
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0]["language_name"], "Rust");

    let frameworks_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/languages/{}/frameworks",
            imported_resume_id,
            languages[0]["id"].as_i64().expect("language id") as i32
        ))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(frameworks_response.status(), Status::Ok);
    let frameworks_body = frameworks_response.into_string().expect("frameworks body");
    let frameworks_json: Value = serde_json::from_str(&frameworks_body).expect("valid json");
    let frameworks = frameworks_json["body"].as_array().expect("array");
    assert_eq!(frameworks.len(), 1);
    assert_eq!(frameworks[0]["framework_name"], "Rocket");
}

#[test]
fn test_work_experience_company_name_with_separator_round_trip() {
    let mut fixture = support::Fixture::new(9_227_015);

    let unique_email = format!(
        "work.separator.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let create_body = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Separator Test",
                "email": unique_email,
                "is_public": true
            })
            .to_string(),
        )
        .dispatch()
        .into_string()
        .expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let resume_id = create_json["body"]["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;
    fixture.track_resume_id(resume_id);

    assert_eq!(
        fixture
            .client()
            .post(format!("/api/resume/{}/work_experiences", resume_id))
            .header(fixture.auth_header())
            .header(ContentType::JSON)
            .body(
                serde_json::json!({
                    "job_title": "Senior Software Engineer",
                    "company_name": "Acme - Global",
                    "start_date": "2020-01-01",
                    "end_date": null,
                    "display_order": 1
                })
                .to_string(),
            )
            .dispatch()
            .status(),
        Status::Created
    );

    let markdown = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch()
        .into_string()
        .expect("markdown body");
    assert!(markdown.contains("### Senior Software Engineer - Acme - Global"));

    let unique_import_email = format!(
        "roundtrip.work.separator.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let markdown_for_import = markdown.replace(
        &format!("- Email: {}", unique_email),
        &format!("- Email: {}", unique_import_email),
    );

    let import_body = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown_for_import)
        .dispatch()
        .into_string()
        .expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let imported_resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(imported_resume_id);

    let work_items_body = fixture
        .client()
        .get(format!(
            "/api/resume/{}/work_experiences",
            imported_resume_id
        ))
        .header(fixture.auth_header())
        .dispatch()
        .into_string()
        .expect("work body");
    let work_items_json: Value = serde_json::from_str(&work_items_body).expect("valid json");
    let work_items = work_items_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["job_title"], "Senior Software Engineer");
    assert_eq!(work_items[0]["company_name"], "Acme - Global");
}

#[test]
fn test_export_public_resume_markdown() {
    let mut fixture = support::Fixture::new(9_227_004);
    let (resume_id, _email) = create_full_resume(&mut fixture, 9_227_001);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .dispatch();

    assert_eq!(export_response.status(), Status::Ok);
    let markdown = export_response.into_string().expect("markdown body");
    assert!(markdown.contains("# John Doe"));
}

#[test]
fn test_export_private_resume_markdown_requires_auth() {
    let mut fixture = support::Fixture::new(9_227_005);

    let unique_email = format!(
        "private.markdown.user.{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let new_resume_json = serde_json::json!({
        "name": "Private User",
        "profile_image_url": null,
        "location": "Secret",
        "email": unique_email,
        "github_url": null,
        "mobile_number": null,
        "is_public": false
    });

    let create_response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(new_resume_json.to_string())
        .dispatch();
    assert_eq!(create_response.status(), Status::Created);

    let create_body = create_response.into_string().expect("Response body");
    let create_json: Value = serde_json::from_str(&create_body).expect("Valid JSON");
    let resume_id = create_json["body"]["id"]
        .as_i64()
        .expect("Resume ID should exist") as i32;
    fixture.track_resume_id(resume_id);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .dispatch();
    assert_eq!(export_response.status(), Status::NotFound);
}

#[test]
fn test_import_resume_markdown_requires_auth() {
    let fixture = support::Fixture::new(9_227_006);

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(markdown_content_type())
        .body(sample_markdown())
        .dispatch();

    assert_eq!(import_response.status(), Status::Unauthorized);
}

#[test]
fn test_import_missing_h1_returns_error() {
    let fixture = support::Fixture::new(9_227_007);
    let markdown = "## Skills\n\n- Rust - 90%".to_string();

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_import_missing_email_returns_error() {
    let fixture = support::Fixture::new(9_227_008);
    let markdown = "# Test\n\n- Location: NYC\n".to_string();

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    let body = response.into_string().expect("body");
    assert!(body.contains("email") || body.contains("Email"));
}

#[test]
fn test_import_invalid_email_format_returns_error() {
    let fixture = support::Fixture::new(9_227_014);
    let markdown = "# Test\n\n- Email: not-an-email\n".to_string();

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    let body = response.into_string().expect("body");
    assert!(body.contains("Invalid email format"));
}

#[test]
fn test_import_malformed_education_heading_returns_error() {
    let fixture = support::Fixture::new(9_227_009);
    let unique_email = unique_markdown_email(9_227_009);
    let markdown = format!(
        "# Test\n\n- Email: {}\n\n## Education\n\n### Bachelor's in CS - University (2020-2024\n- Graduated with honors",
        unique_email
    );

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_import_invalid_skill_percentage_returns_error() {
    let fixture = support::Fixture::new(9_227_010);
    let unique_email = unique_markdown_email(9_227_010);
    let markdown = format!(
        "# Test\n\n- Email: {}\n\n## Skills\n\n- Rust - 150%",
        unique_email
    );

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_import_unknown_section_returns_error() {
    let fixture = support::Fixture::new(9_227_011);
    let unique_email = unique_markdown_email(9_227_011);
    let markdown = format!(
        "# Test\n\n- Email: {}\n\n## Educations\n\n### Bachelor's in CS",
        unique_email
    );

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    let body = response.into_string().expect("body");
    assert!(body.contains("Unknown section") || body.contains("Educations"));
}

#[test]
fn test_import_unicode_characters() {
    let fixture = support::Fixture::new(9_227_012);
    let markdown =
        "# José García\n\n- Email: josé@example.com\n\n## Skills\n\n- Español - 100%".to_string();

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    let body = response.into_string().expect("body");
    let json: Value = serde_json::from_str(&body).expect("json");
    assert_eq!(json["body"]["name"], "José García");
}

#[test]
fn test_import_empty_sections() {
    let mut fixture = support::Fixture::new(9_227_013);
    let unique_email = unique_markdown_email(9_227_013);
    let markdown = format!(
        "# Test\n\n- Email: {}\n\n## Skills\n\n## Education\n\n",
        unique_email
    );

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    let body = response.into_string().expect("body");
    let json: Value = serde_json::from_str(&body).expect("json");
    let resume_id = json["body"]["id"].as_i64().expect("Resume ID should exist") as i32;
    fixture.track_resume_id(resume_id);
}

#[test]
fn test_import_payload_larger_than_default_string_limit_is_accepted() {
    let mut fixture = support::Fixture::new(9_227_016);
    let unique_email = unique_markdown_email(9_227_016);
    let mut markdown = format!("# Test\n\n- Email: {}\n\n## Skills\n\n", unique_email);
    for i in 0..1000 {
        markdown.push_str(&format!("- Skill {} - 50%\n", i));
    }

    assert!(markdown.len() > 8 * 1024);
    assert!(markdown.len() < 1_048_576);

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    let body = response.into_string().expect("body");
    let json: Value = serde_json::from_str(&body).expect("json");
    let resume_id = json["body"]["id"].as_i64().expect("Resume ID should exist") as i32;
    fixture.track_resume_id(resume_id);
}

#[test]
fn test_import_payload_exceeds_1mb_returns_413() {
    let fixture = support::Fixture::new(9_227_017);
    let unique_email = unique_markdown_email(9_227_017);
    let mut markdown = format!("# Test\n\n- Email: {}\n\n## Skills\n\n", unique_email);
    for i in 0..60000 {
        markdown.push_str(&format!("- Skill {} - 50%\n", i));
    }

    assert!(markdown.len() > 1_048_576);

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::PayloadTooLarge);
}

#[test]
fn test_import_child_key_points_are_not_silently_dropped() {
    // V21 invariant: every child row with a valid parent index must be
    // persisted — it must never be silently skipped.  We import a resume
    // with an education entry that has a key point, then read the key points
    // back and assert the count matches what was in the markdown.
    let mut fixture = support::Fixture::new(9_228_001);

    let markdown = "# V21 Test User\n\
                    \n\
                    - Location: Test City\n\
                    - Email: v21.test.oob@example.com\n\
                    \n\
                    ## Education\n\
                    \n\
                    ### Bachelor of Science - State University (Sep 2018 - Jun 2022)\n\
                    - Degree: B.Sc.\n\
                    - Graduated with distinction\n\
                    - Focused on distributed systems\n";

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(import_response.status(), Status::Created);

    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    // Fetch education entries to get the education ID
    let edu_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(edu_response.status(), Status::Ok);
    let edu_body = edu_response.into_string().expect("edu body");
    let edu_json: Value = serde_json::from_str(&edu_body).expect("valid json");
    let edu_list = edu_json["body"].as_array().expect("edu array");
    assert_eq!(edu_list.len(), 1, "should have 1 education entry");

    let edu_id = edu_list[0]["id"].as_i64().expect("edu id") as i32;

    // Fetch key points — must contain exactly the 2 non-metadata bullets
    let kp_response = fixture
        .client()
        .get(format!(
            "/api/resume/{}/education/{}/key_points",
            resume_id, edu_id
        ))
        .header(fixture.auth_header())
        .dispatch();

    assert_eq!(kp_response.status(), Status::Ok);
    let kp_body = kp_response.into_string().expect("kp body");
    let kp_json: Value = serde_json::from_str(&kp_body).expect("valid json");
    let kp_list = kp_json["body"].as_array().expect("kp array");

    // The markdown has 2 key point bullets (the Degree and Description lines
    // are parsed as metadata, not key points).
    assert_eq!(
        kp_list.len(),
        2,
        "education key points must not be silently dropped (V21)"
    );
}

#[test]
fn test_executive_summary_markdown_round_trip() {
    let mut fixture = support::Fixture::new(9_228_007);
    let (resume_id, original_email) = create_full_resume(&mut fixture, 9_228_007);
    let summary = "Systems engineer with distributed-systems experience.";

    let update_response = fixture
        .client()
        .put(format!("/api/resume/{}", resume_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "executive_summary": summary }).to_string())
        .dispatch();
    assert_eq!(update_response.status(), Status::Ok);
    let _ = update_response.into_string();

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export_response.status(), Status::Ok);
    let markdown = export_response.into_string().expect("markdown body");
    assert!(markdown.contains("## Summary"));
    assert!(markdown.contains(summary));

    let imported_email = support::unique_email("exec.summary.markdown");
    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown.replace(
            &format!("- Email: {}", original_email),
            &format!("- Email: {}", imported_email),
        ))
        .dispatch();
    assert_eq!(import_response.status(), Status::Created);

    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid JSON");
    let imported_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(imported_id);
    assert_eq!(import_json["body"]["executive_summary"], summary);
}

#[test]
fn test_executive_summary_markdown_omission() {
    let mut fixture = support::Fixture::new(9_228_008);
    let (resume_id, _email) = create_full_resume(&mut fixture, 9_228_008);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export_response.status(), Status::Ok);
    let markdown = export_response.into_string().expect("markdown body");
    assert!(!markdown.contains("## Summary"));

    let email = support::unique_email("exec.summary.omission");
    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(format!("# No Summary\n\n- Email: {}\n", email))
        .dispatch();
    assert_eq!(import_response.status(), Status::Created);

    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid JSON");
    let imported_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(imported_id);
    assert!(import_json["body"]["executive_summary"].is_null());
}

#[test]
fn test_import_resume_markdown_with_year_only() {
    let mut fixture = support::Fixture::new(9_227_020);
    let unique_email = unique_markdown_email(9_227_020);
    let markdown = format!(
        "# Jane Doe\n\n- Email: {}\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (2020 - 2024)\n- Degree: Bachelor of Science\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (2020 - 2024)\n- Description: Backend development\n",
        unique_email
    );

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["institution_name"], "University of ABC");
    assert_eq!(education_items[0]["degree"], "Bachelor of Science");
    assert_eq!(education_items[0]["start_date"], "2020");
    assert_eq!(education_items[0]["end_date"], "2024");
    assert!(education_items[0].get("start_date_precision").is_none());
    assert!(education_items[0].get("end_date_precision").is_none());

    let work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["job_title"], "Senior Software Engineer");
    assert_eq!(work_items[0]["company_name"], "Tech Corp");
    assert_eq!(work_items[0]["start_date"], "2020");
    assert_eq!(work_items[0]["end_date"], "2024");
    assert!(work_items[0].get("start_date_precision").is_none());
    assert!(work_items[0].get("end_date_precision").is_none());
}

#[test]
fn test_import_resume_markdown_with_invalid_date_range_returns_error() {
    let fixture = support::Fixture::new(9_227_022);
    let unique_email = unique_markdown_email(9_227_022);
    let markdown = format!(
        "# Test\n\n- Email: {}\n\n## Education\n\n### Bachelor's - University of ABC (Sep 2024 - May 2020)",
        unique_email
    );

    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    let body = response.into_string().expect("body");
    assert!(body.contains("date") || body.contains("range"));
}

#[test]
fn test_export_resume_markdown_with_year_only_precision() {
    let mut fixture = support::Fixture::new(9_227_021);
    let unique_email = unique_markdown_email(9_227_021);
    let markdown = format!(
        "# Jane Doe\n\n- Email: {}\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (2020 - 2024)\n- Degree: Bachelor of Science\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (2020 - 2024)\n- Description: Backend development\n",
        unique_email
    );

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();

    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export_response.status(), Status::Ok);
    let exported = export_response.into_string().expect("markdown body");
    assert!(exported.contains(
        "### Bachelor's in Computer Science - University of ABC (2020 - 2024) [order: 0]"
    ));
    assert!(exported.contains("### Senior Software Engineer - Tech Corp (2020 - 2024) [order: 0]"));
    assert!(!exported.contains("2020-01"));
    assert!(!exported.contains("2024-01"));

    let round_trip_email = unique_markdown_email(9_227_024);
    let markdown_for_import = exported.replace(
        &format!("- Email: {}", unique_email),
        &format!("- Email: {}", round_trip_email),
    );

    let reimport_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown_for_import)
        .dispatch();
    assert_eq!(reimport_response.status(), Status::Created);
    let reimport_body = reimport_response.into_string().expect("reimport body");
    let reimport_json: Value = serde_json::from_str(&reimport_body).expect("valid json");
    let reimported_id = reimport_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(reimported_id);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", reimported_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["start_date"], "2020");
    assert_eq!(education_items[0]["end_date"], "2024");

    let work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", reimported_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["start_date"], "2020");
    assert_eq!(work_items[0]["end_date"], "2024");
}

#[test]
fn test_export_resume_markdown_with_month_year_round_trip() {
    let mut fixture = support::Fixture::new(9_227_025);
    let unique_email = unique_markdown_email(9_227_025);
    let markdown = format!(
        "# Jane Doe\n\n- Email: {}\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (September 2020 - May 2024)\n- Degree: Bachelor of Science\n\n## Work Experience\n\n### Senior Software Engineer - Tech Corp (January 1919 - Present)\n- Description: Backend development\n",
        unique_email
    );

    let import_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown)
        .dispatch();
    assert_eq!(import_response.status(), Status::Created);
    let import_body = import_response.into_string().expect("import body");
    let import_json: Value = serde_json::from_str(&import_body).expect("valid json");
    let resume_id = import_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let export_response = fixture
        .client()
        .get(format!("/api/resume/{}/export/markdown", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(export_response.status(), Status::Ok);
    let exported = export_response.into_string().expect("markdown body");
    assert!(exported.contains(
        "### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024) [order: 0]"
    ));
    assert!(
        exported
            .contains("### Senior Software Engineer - Tech Corp (Jan 1919 - Present) [order: 0]")
    );
    assert!(!exported.contains("2020-09-01"));
    assert!(!exported.contains("2024-05-01"));
    assert!(!exported.contains("1919-01-01"));

    let round_trip_email = unique_markdown_email(9_227_026);
    let markdown_for_import = exported.replace(
        &format!("- Email: {}", unique_email),
        &format!("- Email: {}", round_trip_email),
    );

    let reimport_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown_for_import)
        .dispatch();
    assert_eq!(reimport_response.status(), Status::Created);
    let reimport_body = reimport_response.into_string().expect("reimport body");
    let reimport_json: Value = serde_json::from_str(&reimport_body).expect("valid json");
    let reimported_id = reimport_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(reimported_id);

    let education_response = fixture
        .client()
        .get(format!("/api/resume/{}/education", reimported_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(education_response.status(), Status::Ok);
    let education_body = education_response.into_string().expect("education body");
    let education_json: Value = serde_json::from_str(&education_body).expect("valid json");
    let education_items = education_json["body"].as_array().expect("array");
    assert_eq!(education_items.len(), 1);
    assert_eq!(education_items[0]["start_date"], "2020-09");
    assert_eq!(education_items[0]["end_date"], "2024-05");

    let work_response = fixture
        .client()
        .get(format!("/api/resume/{}/work_experiences", reimported_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(work_response.status(), Status::Ok);
    let work_body = work_response.into_string().expect("work body");
    let work_json: Value = serde_json::from_str(&work_body).expect("valid json");
    let work_items = work_json["body"].as_array().expect("array");
    assert_eq!(work_items.len(), 1);
    assert_eq!(work_items[0]["start_date"], "1919-01");
    assert!(work_items[0]["end_date"].is_null());
}

#[test]
fn test_get_markdown_format() {
    let fixture = support::Fixture::new(9_228_010);
    let expected = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../MARKDOWN_FORMAT.md"
    ));

    let response = fixture
        .client()
        .get("/api/resume/markdown-format")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let content_type = response.content_type().expect("content type");
    assert_eq!(content_type.top(), "text");
    assert_eq!(content_type.sub(), "markdown");

    let body = response.into_string().expect("markdown body");
    assert_eq!(body, expected);
}

#[test]
fn test_import_resume_markdown_upserts_existing_by_email() {
    let mut fixture = support::Fixture::new(9_227_030);
    let email = unique_markdown_email(9_227_030);
    let markdown = sample_markdown().replace("jane.doe@example.com", &email);

    let first_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(markdown.clone())
        .dispatch();
    assert_eq!(first_response.status(), Status::Created);
    let first_body = first_response.into_string().expect("first body");
    let first_json: Value = serde_json::from_str(&first_body).expect("valid json");
    let resume_id = first_json["body"]["id"].as_i64().expect("resume id") as i32;
    fixture.track_resume_id(resume_id);

    let updated_markdown = markdown
        .replace("# Jane Doe", "# Jane Updated")
        .replace("- JavaScript - 60%", "- TypeScript - 70%");

    let second_response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(markdown_content_type())
        .body(updated_markdown)
        .dispatch();
    assert_eq!(second_response.status(), Status::Ok);
    let second_body = second_response.into_string().expect("second body");
    let second_json: Value = serde_json::from_str(&second_body).expect("valid json");
    assert_eq!(second_json["body"]["id"], resume_id);
    assert_eq!(second_json["body"]["name"], "Jane Updated");

    let skills_response = fixture
        .client()
        .get(format!("/api/resume/{}/skills", resume_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(skills_response.status(), Status::Ok);
    let skills_body = skills_response.into_string().expect("skills body");
    let skills_json: Value = serde_json::from_str(&skills_body).expect("valid json");
    let skills = skills_json["body"].as_array().expect("array");
    assert!(
        skills
            .iter()
            .any(|s| s["skill_name"] == "TypeScript" && s["confidence_percentage"] == 70)
    );
    assert!(!skills.iter().any(|s| s["skill_name"] == "JavaScript"));
}
