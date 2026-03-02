use diesel::prelude::*;
use domain::models::{NewSkill, NewSkillRequest, Resume, Skill};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn create_skill(
    user_id_value: i32,
    resume_id_value: i32,
    payload: Json<NewSkillRequest>,
) -> Result<Created<String>, Custom<String>> {
    use domain::schema::resumes;
    use domain::schema::skills;

    let existing_resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Resume with id {} not found", resume_id_value),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match existing_resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response::<String> {
                body: "Forbidden".to_string(),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let payload = payload.into_inner();
    let new_skill = NewSkill {
        resume_id: resume_id_value,
        skill_name: payload.skill_name,
        confidence_percentage: payload.confidence_percentage,
        display_order: payload.display_order,
    };

    match diesel::insert_into(skills::table)
        .values(&new_skill)
        .get_result::<Skill>(&mut establish_connection())
    {
        Ok(skill) => {
            let response = Response::<Skill> { body: skill };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => {
            panic!("Database error - {}", err);
        }
    }
}
