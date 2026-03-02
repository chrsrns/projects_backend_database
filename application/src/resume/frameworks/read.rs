use diesel::prelude::*;
use domain::models::{Framework, Language, Resume};
use infrastructure::establish_connection;
use rocket::response::status::NotFound;
use shared::response_models::Response;

pub fn list_frameworks(
    resume_id_value: i32,
    language_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<String, NotFound<String>> {
    use domain::schema::frameworks::dsl as frameworks_dsl;
    use domain::schema::languages::dsl as languages_dsl;
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    let _resume: Resume = match resume_query.first(&mut establish_connection()) {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Resume with id {} not found", resume_id_value),
            };
            return Err(NotFound(serde_json::to_string(&response).unwrap()));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let language: Language = match languages_dsl::languages
        .filter(languages_dsl::id.eq(language_id_value))
        .filter(languages_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(l) => l,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: "Language not found".to_string(),
            };
            return Err(NotFound(serde_json::to_string(&response).unwrap()));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let mut items: Vec<Framework> = match frameworks_dsl::frameworks
        .filter(frameworks_dsl::language_id.eq(language.id))
        .load::<Framework>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => panic!("Database error - {}", err),
    };

    items.sort_by_key(|f| (f.display_order.unwrap_or(0), f.id));

    let response = Response::<Vec<Framework>> { body: items };

    Ok(serde_json::to_string(&response).unwrap())
}
