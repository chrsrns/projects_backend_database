use rocket::http::{ContentType, Status};
use serde_json::Value;

mod support;

fn create_resume(
    fixture: &mut support::Fixture,
    name: &str,
    email: &str,
    visibility: &str,
) -> uuid::Uuid {
    let response = fixture
        .client()
        .post("/api/new_resume")
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "name": name,
                "email": email,
                "visibility": visibility,
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Created, "resume create {name}");
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let id = support::resume_uuid(&json["body"]["id"]);
    fixture.track_resume_id(id);
    id
}

fn list(fixture: &support::Fixture, query: &str) -> (Status, Value) {
    let response = fixture
        .client()
        .get(format!("/api/resumes{query}"))
        .header(fixture.auth_header())
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
}

fn items(page: &Value) -> &Vec<Value> {
    page["body"]["items"].as_array().expect("items array")
}

/// Every seeded row carries the fixture seed as a searchable marker so
/// `?search=<seed>` narrows the shared test database to this fixture's rows.
fn seed(fixture: &mut support::Fixture, count: usize, marker: u64) -> Vec<uuid::Uuid> {
    (0..count)
        .map(|i| {
            create_resume(
                fixture,
                &format!("SEED{i:02} M{marker}"),
                &format!("page.seed.{i}.{marker}@example.com"),
                "private",
            )
        })
        .collect()
}

#[test]
fn defaults_echo_limit_offset_and_id_order() {
    let mut fixture = support::Fixture::new(9_250_001);
    let mut ids = seed(&mut fixture, 3, 9_250_001);
    ids.sort();

    let (status, page) = list(&fixture, "?search=9250001");
    assert_eq!(status, Status::Ok);
    assert_eq!(page["body"]["limit"].as_i64(), Some(20));
    assert_eq!(page["body"]["offset"].as_i64(), Some(0));
    assert_eq!(page["body"]["total"].as_i64(), Some(3));

    let got: Vec<String> = items(&page)
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    let want: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    assert_eq!(got, want, "default order is id ascending");
}

#[test]
fn limit_offset_window_and_total() {
    let mut fixture = support::Fixture::new(9_250_002);
    let mut ids = seed(&mut fixture, 5, 9_250_002);
    ids.sort();

    let (status, page) = list(&fixture, "?search=9250002&limit=2&offset=2");
    assert_eq!(status, Status::Ok);
    assert_eq!(page["body"]["total"].as_i64(), Some(5));
    assert_eq!(page["body"]["limit"].as_i64(), Some(2));
    assert_eq!(page["body"]["offset"].as_i64(), Some(2));

    let items = items(&page);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"].as_str().unwrap(), ids[2].to_string());
    assert_eq!(items[1]["id"].as_str().unwrap(), ids[3].to_string());
}

#[test]
fn invalid_params_are_400() {
    let mut fixture = support::Fixture::new(9_250_003);
    let _ids = seed(&mut fixture, 1, 9_250_003);

    for query in [
        "?limit=0",
        "?limit=101",
        "?limit=-1",
        "?limit=abc",
        "?limit=",
        "?limit=1&limit=2",
        "?offset=-1",
        "?offset=xyz",
        "?sort=bogus",
        "?order=sideways",
        "?limt=5",
        "?wat=1",
    ] {
        let (status, _) = list(&fixture, query);
        assert_eq!(status, Status::BadRequest, "query {query} should be 400");
    }
}

#[test]
fn sort_name_asc_then_id_tiebreak() {
    let mut fixture = support::Fixture::new(9_250_004);
    let dup_a = create_resume(
        &mut fixture,
        "AAAA M9250004",
        "tie.a.9250004@example.com",
        "private",
    );
    let dup_b = create_resume(
        &mut fixture,
        "AAAA M9250004",
        "tie.b.9250004@example.com",
        "private",
    );
    let _other = create_resume(
        &mut fixture,
        "ZZZZ M9250004",
        "tie.c.9250004@example.com",
        "private",
    );

    let (status, page) = list(&fixture, "?search=9250004&sort=name&order=asc");
    assert_eq!(status, Status::Ok);
    let items = items(&page);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["name"].as_str().unwrap(), "AAAA M9250004");
    assert_eq!(items[1]["name"].as_str().unwrap(), "AAAA M9250004");
    assert_eq!(items[2]["name"].as_str().unwrap(), "ZZZZ M9250004");

    let mut tied = [dup_a, dup_b];
    tied.sort();
    let got: Vec<String> = items[0..2]
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        got,
        tied.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        "equal names fall back to id ascending"
    );
}

#[test]
fn sort_updated_at_desc() {
    let mut fixture = support::Fixture::new(9_250_005);
    let first = create_resume(
        &mut fixture,
        "FIRST M9250005",
        "upd.a.9250005@example.com",
        "private",
    );
    let second = create_resume(
        &mut fixture,
        "SECOND M9250005",
        "upd.b.9250005@example.com",
        "private",
    );

    let (status, page) = list(&fixture, "?search=9250005&sort=updated_at&order=desc");
    assert_eq!(status, Status::Ok);
    let items = items(&page);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"].as_str().unwrap(), second.to_string());
    assert_eq!(items[1]["id"].as_str().unwrap(), first.to_string());
}

#[test]
fn search_and_terms_and_fields() {
    let mut fixture = support::Fixture::new(9_250_006);
    let base = create_resume(
        &mut fixture,
        "SEARCHABLE M9250006",
        "search.a.9250006@example.com",
        "private",
    );
    let _noise = create_resume(
        &mut fixture,
        "OTHER M9250006",
        "search.b.9250006@example.com",
        "private",
    );

    let variant_resp = fixture
        .client()
        .post(format!("/api/resume/{base}/variants"))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(
            serde_json::json!({
                "company_name": "NEEDLECORP9250006",
                "job_description": "rust backend needle",
            })
            .to_string(),
        )
        .dispatch();
    assert_eq!(variant_resp.status(), Status::Created);
    let variant_json: Value = serde_json::from_str(&variant_resp.into_string().unwrap()).unwrap();
    fixture.track_resume_id(support::resume_uuid(&variant_json["body"]["id"]));

    let (status, page) = list(&fixture, "?search=searchable+9250006");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(2),
        "base and variant share the cloned name"
    );

    let (status, page) = list(&fixture, "?search=needlecorp9250006+backend");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(1),
        "both terms must hit the variant's company and description"
    );

    let (status, page) = list(&fixture, "?search=needlecorp9250006+nomatch");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(0),
        "AND-of-terms fails"
    );
}

#[test]
fn search_escapes_like_metachars() {
    let mut fixture = support::Fixture::new(9_250_007);
    let _ = seed(&mut fixture, 3, 9_250_007);

    let (status, page) = list(&fixture, "?search=9250007+%25");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(0),
        "a literal percent is not a wildcard"
    );

    let (status, page) = list(&fixture, "?search=9250007+_");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(0),
        "a literal underscore is not a single-char wildcard"
    );
}

#[test]
fn search_blank_is_unfiltered() {
    let mut fixture = support::Fixture::new(9_250_008);
    let ids = seed(&mut fixture, 2, 9_250_008);

    for query in ["?search=", "?search=%20%20"] {
        let (status, page) = list(&fixture, query);
        assert_eq!(status, Status::Ok, "blank search {query}");
        assert!(
            page["body"]["total"].as_i64().unwrap() >= 2,
            "blank search {query} must not filter own rows out"
        );
    }

    let (_, page) = list(&fixture, "?search=%20%20");
    let found: Vec<String> = items(&page)
        .iter()
        .filter_map(|r| r["id"].as_str().map(str::to_string))
        .collect();
    assert!(
        ids.iter().any(|id| found.contains(&id.to_string())),
        "own rows are reachable through a blank search"
    );
}

#[test]
fn offset_past_end_is_empty_page() {
    let mut fixture = support::Fixture::new(9_250_009);
    let _ = seed(&mut fixture, 2, 9_250_009);

    let (status, page) = list(&fixture, "?search=9250009&offset=99");
    assert_eq!(status, Status::Ok);
    assert_eq!(page["body"]["total"].as_i64(), Some(2));
    assert_eq!(items(&page).len(), 0);
}

#[test]
fn scope_unpierced_for_non_owner() {
    let mut fixture = support::Fixture::new(9_250_010);
    let _ = create_resume(
        &mut fixture,
        "OWNERONLY9250010",
        "scope.a.9250010@example.com",
        "private",
    );
    let public_id = create_resume(
        &mut fixture,
        "PUBLICROW9250010",
        "scope.b.9250010@example.com",
        "public",
    );

    let anon = fixture
        .client()
        .get("/api/resumes?search=owneronly9250010")
        .dispatch();
    assert_eq!(anon.status(), Status::Ok);
    let json: Value = serde_json::from_str(&anon.into_string().unwrap()).unwrap();
    assert_eq!(
        json["body"]["total"].as_i64(),
        Some(0),
        "search stays inside the caller's scope"
    );

    let anon = fixture
        .client()
        .get("/api/resumes?search=publicrow9250010")
        .dispatch();
    assert_eq!(anon.status(), Status::Ok);
    let json: Value = serde_json::from_str(&anon.into_string().unwrap()).unwrap();
    let items = items(&json);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"].as_str().unwrap(), public_id.to_string());
}
