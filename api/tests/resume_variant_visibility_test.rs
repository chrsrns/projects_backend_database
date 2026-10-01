use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

const RESUME_VIEW_KEYS: [&str; 22] = [
    "id",
    "name",
    "profile_image_url",
    "location",
    "email",
    "github_url",
    "mobile_number",
    "created_at",
    "updated_at",
    "created_by",
    "is_public",
    "executive_summary",
    "video",
    "is_variant",
    "base_resume_id",
    "show_variant_tag",
    "company_name",
    "role_title",
    "target_date",
    "target_date_precision",
    "job_description",
    "variant_label",
];

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn markdown(email: &str, public: bool) -> String {
    format!(
        "# Jane Doe\n\n- Location: New York, NY\n- Email: {}\n- GitHub: https://github.com/janedoe\n- Mobile: +1987654321\n- Public: {}\n\n## Skills\n\n- Rust - 90%\n",
        email, public
    )
}

fn assert_flat_resume_view(value: &Value) {
    let object = value.as_object().expect("resume payload is an object");
    assert!(
        object.get("resume").is_none(),
        "the payload must not nest a resume object"
    );
    for key in RESUME_VIEW_KEYS {
        assert!(
            object.contains_key(key),
            "missing top-level field {} in {}",
            key,
            serde_json::to_string(value).unwrap()
        );
    }
}

fn import_base(fixture: &support::Fixture, lock_key: i64, public: bool) -> (Status, Value) {
    let email = format!("variant.view.{}.{}@example.com", lock_key, unique_suffix());
    let response = fixture
        .client()
        .post("/api/resume/import/markdown")
        .header(fixture.auth_header())
        .header(ContentType::new("text", "markdown"))
        .body(markdown(&email, public))
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn post_variant(fixture: &support::Fixture, base_id: i32, body: Value) -> (Status, Value) {
    let response = fixture
        .client()
        .post(format!("/api/resume/{}/variants", base_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn get_resume(fixture: &support::Fixture, resume_id: i32, token: Option<&str>) -> (Status, Value) {
    let request = fixture.client().get(format!("/api/resume/{}", resume_id));
    let request = match token {
        Some(token) => request.header(support::auth_header(token)),
        None => request,
    };
    let response = request.dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

#[test]
fn resume_view_shape_flat_across_all_endpoints() {
    let mut fixture = support::Fixture::new(9_241_030);

    let (status, imported) = import_base(&fixture, 9_241_030, true);
    assert_eq!(status, Status::Created);
    let base_id = imported["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);
    assert_flat_resume_view(&imported["body"]);

    let created = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": "Flat Shape",
                "email": format!("variant.flat.{}@example.com", unique_suffix()),
                "is_public": true
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(created.status(), Status::Created);
    let created_json: Value = serde_json::from_str(&created.into_string().unwrap()).unwrap();
    assert_flat_resume_view(&created_json["body"]);
    let created_id = created_json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(created_id);

    let updated = fixture
        .client()
        .put(format!("/api/resume/{}", created_id))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(serde_json::json!({ "location": "Moved" }).to_string())
        .dispatch();
    assert_eq!(updated.status(), Status::Ok);
    let updated_json: Value = serde_json::from_str(&updated.into_string().unwrap()).unwrap();
    assert_flat_resume_view(&updated_json["body"]);

    let (status, variant_json) = post_variant(
        &fixture,
        base_id,
        serde_json::json!({ "company_name": "Acme Corp", "target_date": "2026-03" }),
    );
    assert_eq!(status, Status::Created);
    assert_flat_resume_view(&variant_json["body"]);
    let variant_id = variant_json["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(variant_id);

    let (status, single) = get_resume(&fixture, base_id, Some(fixture.auth_token()));
    assert_eq!(status, Status::Ok);
    assert_flat_resume_view(&single["body"]);

    let listed = fixture
        .client()
        .get("/api/resumes")
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(listed.status(), Status::Ok);
    let listed_json: Value = serde_json::from_str(&listed.into_string().unwrap()).unwrap();
    let items = listed_json["body"].as_array().unwrap();
    assert!(!items.is_empty());
    for item in items {
        assert_flat_resume_view(item);
    }

    let variants = fixture
        .client()
        .get(format!("/api/resume/{}/variants", base_id))
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(variants.status(), Status::Ok);
    let variants_json: Value = serde_json::from_str(&variants.into_string().unwrap()).unwrap();
    let variant_items = variants_json["body"].as_array().unwrap();
    assert_eq!(variant_items.len(), 1);
    assert_flat_resume_view(&variant_items[0]);
}

#[test]
fn list_resumes_includes_variants_in_id_order() {
    let mut fixture = support::Fixture::new(9_241_033);

    let (status, base) = import_base(&fixture, 9_241_033, true);
    assert_eq!(status, Status::Created);
    let base_id = base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(base_id);

    let mut variant_ids = Vec::new();
    for company in ["Acme Corp", "Globex", "Initech"] {
        let (status, json) = post_variant(
            &fixture,
            base_id,
            serde_json::json!({ "company_name": company, "is_public": true }),
        );
        assert_eq!(status, Status::Created);
        let id = json["body"]["id"].as_i64().unwrap() as i32;
        fixture.track_resume_id(id);
        variant_ids.push(id);
    }

    let response = fixture
        .client()
        .get("/api/resumes")
        .header(fixture.auth_header())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let items = json["body"].as_array().unwrap();

    let ids: Vec<i32> = items
        .iter()
        .map(|item| item["id"].as_i64().unwrap() as i32)
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "the list keeps its ascending id order");

    for id in std::iter::once(base_id).chain(variant_ids.iter().copied()) {
        assert!(
            ids.contains(&id),
            "GET /resumes should include resume {}",
            id
        );
    }

    for variant_id in &variant_ids {
        let item = items
            .iter()
            .find(|item| item["id"].as_i64().unwrap() as i32 == *variant_id)
            .expect("variant present in the list");
        assert_eq!(item["is_variant"], Value::Bool(true));
        assert_eq!(
            item["base_resume_id"].as_i64().unwrap() as i32,
            base_id,
            "the owner sees the parent link"
        );
    }

    let base_item = items
        .iter()
        .find(|item| item["id"].as_i64().unwrap() as i32 == base_id)
        .expect("base present in the list");
    assert_eq!(base_item["is_variant"], Value::Bool(false));
    assert!(base_item["base_resume_id"].is_null());
}

#[test]
fn resume_view_redacts_base_and_tag_for_other_viewers() {
    let mut fixture = support::Fixture::new(9_241_031);

    let (status, public_base) = import_base(&fixture, 9_241_031, true);
    assert_eq!(status, Status::Created);
    let public_base_id = public_base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(public_base_id);

    let (status, public_variant) = post_variant(
        &fixture,
        public_base_id,
        serde_json::json!({
            "company_name": "Acme Corp",
            "is_public": true,
            "show_variant_tag": false
        }),
    );
    assert_eq!(status, Status::Created);
    let public_variant_id = public_variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(public_variant_id);

    let (status, private_base) = import_base(&fixture, 9_241_032, false);
    assert_eq!(status, Status::Created);
    let private_base_id = private_base["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(private_base_id);

    let (status, tagged_variant) = post_variant(
        &fixture,
        private_base_id,
        serde_json::json!({
            "company_name": "Globex",
            "is_public": true,
            "show_variant_tag": true
        }),
    );
    assert_eq!(status, Status::Created);
    let tagged_variant_id = tagged_variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(tagged_variant_id);

    let (status, quiet_variant) = post_variant(
        &fixture,
        private_base_id,
        serde_json::json!({
            "company_name": "Initech",
            "is_public": true,
            "show_variant_tag": false
        }),
    );
    assert_eq!(status, Status::Created);
    let quiet_variant_id = quiet_variant["body"]["id"].as_i64().unwrap() as i32;
    fixture.track_resume_id(quiet_variant_id);

    let other = support::register_and_login(fixture.client(), "variant.view.other");
    fixture.track_user_id(other.user_id);
    fixture.track_session_id(other.token.clone());

    // Owner: every field is visible, including the tag.
    let (status, owner_view) = get_resume(&fixture, public_variant_id, Some(fixture.auth_token()));
    assert_eq!(status, Status::Ok);
    assert_eq!(owner_view["body"]["is_variant"], Value::Bool(true));
    assert_eq!(
        owner_view["body"]["base_resume_id"].as_i64().unwrap() as i32,
        public_base_id
    );
    assert_eq!(owner_view["body"]["show_variant_tag"], Value::Bool(false));

    // Another user: the base is public, so the variant status and the
    // backlink survive, but the tag itself is owner-only.
    let (status, other_view) = get_resume(&fixture, public_variant_id, Some(&other.token));
    assert_eq!(status, Status::Ok);
    assert_eq!(other_view["body"]["is_variant"], Value::Bool(true));
    assert_eq!(
        other_view["body"]["base_resume_id"].as_i64().unwrap() as i32,
        public_base_id
    );
    assert!(other_view["body"]["show_variant_tag"].is_null());

    // Anonymous sees the same as any other viewer.
    let (status, anonymous_view) = get_resume(&fixture, public_variant_id, None);
    assert_eq!(status, Status::Ok);
    assert_eq!(anonymous_view["body"]["is_variant"], Value::Bool(true));
    assert!(anonymous_view["body"]["show_variant_tag"].is_null());

    // Private base, tag on: the variant still announces itself, but the
    // base id is redacted.
    let (status, tagged_view) = get_resume(&fixture, tagged_variant_id, Some(&other.token));
    assert_eq!(status, Status::Ok);
    assert_eq!(tagged_view["body"]["is_variant"], Value::Bool(true));
    assert!(tagged_view["body"]["base_resume_id"].is_null());

    // Private base, tag off: nothing about the variant status leaks.
    let (status, quiet_view) = get_resume(&fixture, quiet_variant_id, Some(&other.token));
    assert_eq!(status, Status::Ok);
    assert_eq!(quiet_view["body"]["is_variant"], Value::Bool(false));
    assert!(quiet_view["body"]["base_resume_id"].is_null());
    assert!(quiet_view["body"]["show_variant_tag"].is_null());

    // The owner still sees the private base as the variant's parent.
    let (status, owner_quiet) = get_resume(&fixture, quiet_variant_id, Some(fixture.auth_token()));
    assert_eq!(status, Status::Ok);
    assert_eq!(owner_quiet["body"]["is_variant"], Value::Bool(true));
    assert_eq!(
        owner_quiet["body"]["base_resume_id"].as_i64().unwrap() as i32,
        private_base_id
    );

    // The private base itself stays invisible to the other user.
    let (status, _) = get_resume(&fixture, private_base_id, Some(&other.token));
    assert_eq!(status, Status::NotFound);
}
