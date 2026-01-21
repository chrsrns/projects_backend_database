use diesel::prelude::*;
use domain::models::{Language, Resume};
use infrastructure::establish_connection;
use rocket::response::status::NotFound;
use shared::response_models::{Response, ResponseBody};

pub fn list_languages(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<String, NotFound<String>> {
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
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Resume with id {} not found",
                    resume_id_value
                )),
            };
            return Err(NotFound(serde_json::to_string(&response).unwrap()));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let mut items: Vec<Language> = match languages_dsl::languages
        .filter(languages_dsl::resume_id.eq(resume_id_value))
        .load::<Language>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => panic!("Database error - {}", err),
    };

    items.sort_by_key(|l| (l.display_order.unwrap_or(0), l.id));

    let response = Response {
        body: ResponseBody::Languages(items),
    };

    Ok(serde_json::to_string(&response).unwrap())
}
