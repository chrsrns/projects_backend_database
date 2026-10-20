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

/// Page-unit accessors for the grouped shape: every item is
/// `{base: ResumeView, variants: ResumeView[]}`.
fn unit_id(unit: &Value) -> String {
    unit["base"]["id"]
        .as_str()
        .expect("unit base id")
        .to_string()
}

fn unit_variants(unit: &Value) -> &Vec<Value> {
    unit["variants"].as_array().expect("unit variants array")
}

/// Every resume id visible inside a group: the unit's own id plus each
/// nested variant id. Absence checks must descend into both.
fn group_ids(unit: &Value) -> Vec<String> {
    let mut ids = vec![unit_id(unit)];
    ids.extend(
        unit_variants(unit)
            .iter()
            .map(|v| v["id"].as_str().unwrap().to_string()),
    );
    ids
}

fn create_variant(
    fixture: &mut support::Fixture,
    base: uuid::Uuid,
    body: serde_json::Value,
) -> uuid::Uuid {
    let response = fixture
        .client()
        .post(format!("/api/resume/{base}/variants"))
        .header(fixture.auth_header())
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::Created, "variant create");
    let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let id = support::resume_uuid(&json["body"]["id"]);
    fixture.track_resume_id(id);
    id
}

fn list_anon(fixture: &support::Fixture, query: &str) -> (Status, Value) {
    let response = fixture
        .client()
        .get(format!("/api/resumes{query}"))
        .dispatch();
    let status = response.status();
    let json = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    (status, json)
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
        .map(|r| r["base"]["id"].as_str().unwrap().to_string())
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
    assert_eq!(items[0]["base"]["id"].as_str().unwrap(), ids[2].to_string());
    assert_eq!(items[1]["base"]["id"].as_str().unwrap(), ids[3].to_string());
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
    assert_eq!(items[0]["base"]["name"].as_str().unwrap(), "AAAA M9250004");
    assert_eq!(items[1]["base"]["name"].as_str().unwrap(), "AAAA M9250004");
    assert_eq!(items[2]["base"]["name"].as_str().unwrap(), "ZZZZ M9250004");

    let mut tied = [dup_a, dup_b];
    tied.sort();
    let got: Vec<String> = items[0..2]
        .iter()
        .map(|r| r["base"]["id"].as_str().unwrap().to_string())
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
    assert_eq!(items[0]["base"]["id"].as_str().unwrap(), second.to_string());
    assert_eq!(items[1]["base"]["id"].as_str().unwrap(), first.to_string());
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
        Some(1),
        "base and variant nest into one group"
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
        .filter_map(|r| r["base"]["id"].as_str().map(str::to_string))
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
    assert_eq!(
        items[0]["base"]["id"].as_str().unwrap(),
        public_id.to_string()
    );
}

#[test]
fn variant_nests_under_base_in_one_unit() {
    let mut fixture = support::Fixture::new(9_250_011);
    let base = create_resume(
        &mut fixture,
        "NESTBASE M9250011",
        "nest.a.9250011@example.com",
        "private",
    );
    let variant = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"company_name": "NESTCORP9250011"}),
    );

    let (status, page) = list(&fixture, "?search=9250011");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(1),
        "base + attached variant = one page unit"
    );

    let units = items(&page);
    assert_eq!(units.len(), 1);
    let unit = &units[0];
    assert_eq!(unit_id(unit), base.to_string());
    let variants = unit_variants(unit);
    assert_eq!(variants.len(), 1);
    assert_eq!(variants[0]["id"].as_str().unwrap(), variant.to_string());
    assert_eq!(
        variants[0]["base_resume_id"].as_str().unwrap(),
        base.to_string(),
        "nested variant carries the backlink to its base"
    );

    let top_level: Vec<String> = items(&page).iter().map(unit_id).collect();
    assert!(
        !top_level.contains(&variant.to_string()),
        "attached variant is absent as a top-level unit"
    );
}

#[test]
fn group_variants_ordered_id_ascending() {
    let mut fixture = support::Fixture::new(9_250_012);
    let base = create_resume(
        &mut fixture,
        "ORDBASE M9250012",
        "ord.a.9250012@example.com",
        "private",
    );
    let first = create_variant(&mut fixture, base, serde_json::json!({}));
    let second = create_variant(&mut fixture, base, serde_json::json!({}));
    let mut want = vec![first.to_string(), second.to_string()];
    want.sort();

    let (status, page) = list(&fixture, "?search=9250012");
    assert_eq!(status, Status::Ok);
    let variants = unit_variants(&items(&page)[0]);
    let got: Vec<String> = variants
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(got, want, "group variants ordered id ascending");
}

#[test]
fn standalone_variant_of_invisible_base() {
    let mut fixture = support::Fixture::new(9_250_013);
    let base = create_resume(
        &mut fixture,
        "HIDDENBASE9250013",
        "hidden.9250013@example.com",
        "private",
    );
    let tagged = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"visibility": "public", "show_variant_tag": true}),
    );
    let untagged = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"visibility": "public", "show_variant_tag": false}),
    );

    let (status, page) = list_anon(&fixture, "?limit=100");
    assert_eq!(status, Status::Ok);
    let items = items(&page);

    let tagged_unit = items
        .iter()
        .find(|u| unit_id(u) == tagged.to_string())
        .expect("tagged standalone variant unit");
    assert_eq!(tagged_unit["base"]["is_variant"].as_bool(), Some(true));
    assert!(tagged_unit["base"]["base_resume_id"].is_null());
    assert!(unit_variants(tagged_unit).is_empty());

    let untagged_unit = items
        .iter()
        .find(|u| unit_id(u) == untagged.to_string())
        .expect("untagged standalone variant unit");
    assert_eq!(untagged_unit["base"]["is_variant"].as_bool(), Some(false));
    assert!(untagged_unit["base"]["base_resume_id"].is_null());
    assert!(unit_variants(untagged_unit).is_empty());
}

#[test]
fn invisible_variants_absent_from_group() {
    let mut fixture = support::Fixture::new(9_250_014);
    let base = create_resume(
        &mut fixture,
        "PUBBASE M9250014",
        "pub.a.9250014@example.com",
        "public",
    );
    let private_variant = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"visibility": "private", "company_name": "PRIVCO9250014"}),
    );
    let unlisted_variant = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"visibility": "unlisted", "company_name": "UNLCO9250014"}),
    );
    let public_variant = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"visibility": "public", "company_name": "PUBCO9250014"}),
    );

    let (status, page) = list_anon(&fixture, "?search=9250014");
    assert_eq!(status, Status::Ok);
    let items = items(&page);
    assert_eq!(items.len(), 1);
    let unit = &items[0];
    assert_eq!(unit_id(unit), base.to_string());

    let visible: Vec<String> = unit_variants(unit)
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        visible,
        vec![public_variant.to_string()],
        "only caller-visible variants nest; private/unlisted appear nowhere"
    );
    assert!(!visible.contains(&private_variant.to_string()));
    assert!(!visible.contains(&unlisted_variant.to_string()));

    let top_level: Vec<String> = items.iter().map(unit_id).collect();
    assert!(!top_level.contains(&private_variant.to_string()));
    assert!(!top_level.contains(&unlisted_variant.to_string()));
}

#[test]
fn search_variant_hit_pulls_group_with_all_variants() {
    let mut fixture = support::Fixture::new(9_250_015);
    let base = create_resume(
        &mut fixture,
        "GROUPBASE M9250015",
        "grp.a.9250015@example.com",
        "private",
    );
    let _hit = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"company_name": "HITCORP9250015"}),
    );
    let _miss = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"company_name": "OTHERCO9250015"}),
    );

    let (status, page) = list(&fixture, "?search=hitcorp9250015");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(1),
        "variant-only hit qualifies the group as one unit"
    );
    let unit = &items(&page)[0];
    assert_eq!(unit_id(unit), base.to_string());
    assert_eq!(
        unit_variants(unit).len(),
        2,
        "a qualifying group carries all caller-visible variants"
    );
}

#[test]
fn search_two_matching_variants_one_unit() {
    let mut fixture = support::Fixture::new(9_250_016);
    let base = create_resume(
        &mut fixture,
        "DEDUPBASE M9250016",
        "dup.a.9250016@example.com",
        "private",
    );
    let _v1 = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"company_name": "SHAREDCO9250016"}),
    );
    let _v2 = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"company_name": "SHAREDCO9250016"}),
    );

    let (status, page) = list(&fixture, "?search=sharedco9250016");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(1),
        "two matching variants of one base dedup to one unit"
    );
    assert_eq!(items(&page).len(), 1);
    assert_eq!(unit_variants(&items(&page)[0]).len(), 2);
}

#[test]
fn search_terms_split_across_rows_do_not_qualify() {
    let mut fixture = support::Fixture::new(9_250_017);
    let base = create_resume(
        &mut fixture,
        "SPLITBASE M9250017",
        "split.a.9250017@example.com",
        "private",
    );
    let v1 = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"company_name": "SPLITCORP9250017"}),
    );
    let v2 = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"role_title": "SPLITROLE9250017"}),
    );

    // Variants clone the base name; rename both so no variant row can
    // carry the base term, keeping the split-row assertions honest.
    for (variant, name) in [(v1, "V1NEUTRAL9250017"), (v2, "V2NEUTRAL9250017")] {
        let resp = fixture
            .client()
            .put(format!("/api/resume/{variant}"))
            .header(fixture.auth_header())
            .header(ContentType::JSON)
            .body(serde_json::json!({"name": name}).to_string())
            .dispatch();
        assert_eq!(resp.status(), Status::Ok, "rename variant {variant}");
    }

    let (status, page) = list(&fixture, "?search=splitbase+splitcorp9250017");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(0),
        "terms split across base and variant rows do not qualify"
    );

    let (status, page) = list(&fixture, "?search=splitcorp9250017+splitrole9250017");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(0),
        "terms split across two variant rows do not qualify"
    );
}

#[test]
fn search_invisible_variant_term_qualifies_nothing() {
    let mut fixture = support::Fixture::new(9_250_018);
    let base = create_resume(
        &mut fixture,
        "GHOSTBASE M9250018",
        "ghost.a.9250018@example.com",
        "public",
    );
    let _ghost = create_variant(
        &mut fixture,
        base,
        serde_json::json!({"visibility": "private", "company_name": "GHOSTCO9250018"}),
    );

    let (status, page) = list_anon(&fixture, "?search=ghostco9250018");
    assert_eq!(status, Status::Ok);
    assert_eq!(
        page["body"]["total"].as_i64(),
        Some(0),
        "a term matching only a caller-invisible variant qualifies no unit"
    );
}

#[test]
fn group_ids_covers_nested_absence_checks() {
    let mut fixture = support::Fixture::new(9_250_019);
    let base = create_resume(
        &mut fixture,
        "HELPERBASE M9250019",
        "hlp.a.9250019@example.com",
        "private",
    );
    let variant = create_variant(&mut fixture, base, serde_json::json!({}));

    let (status, page) = list(&fixture, "?search=9250019");
    assert_eq!(status, Status::Ok);
    let all: Vec<String> = items(&page).iter().flat_map(group_ids).collect();
    assert!(all.contains(&base.to_string()));
    assert!(all.contains(&variant.to_string()));
}
