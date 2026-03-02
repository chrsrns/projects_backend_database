use diesel::prelude::*;
use domain::models::{NewResume, NewResumeRequest, Resume};
use infrastructure::establish_connection;
use rocket::response::status::{Conflict, Created};
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn create_resume(
    user_id_value: i32,
    resume: Json<NewResumeRequest>,
) -> Result<Created<String>, Conflict<String>> {
    use domain::schema::resumes;

    let resume = resume.into_inner();
    let new_resume = NewResume {
        name: resume.name,
        profile_image_url: resume.profile_image_url,
        location: resume.location,
        email: resume.email,
        github_url: resume.github_url,
        mobile_number: resume.mobile_number,
        created_by: Some(user_id_value),
        is_public: resume.is_public.unwrap_or(false),
    };

    match diesel::insert_into(resumes::table)
        .values(&new_resume)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(resume) => {
            let response = Response::<Resume> { body: resume };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => {
                let response = Response::<String> {
                    body: "Resume with this email already exists".to_string(),
                };
                Err(Conflict(serde_json::to_string(&response).unwrap()))
            }
            _ => panic!("Database error - {}", err),
        },
    }
}
