use diesel::prelude::*;
use domain::models::{Resume, Skill, UpdateSkill};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::{Response, ResponseBody};

pub fn update_skill(
    user_id_value: i32,
    skill_id_value: i32,
    payload: Json<UpdateSkill>,
) -> Result<String, Custom<String>> {
    use domain::schema::resumes;
    use domain::schema::skills;

    let existing: Skill = match skills::table
        .find(skill_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!("Skill with id {} not found", skill_id_value)),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Resume not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response {
                body: ResponseBody::Message("Forbidden".to_string()),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let payload = payload.into_inner();

    match diesel::update(skills::table.find(skill_id_value))
        .set(&payload)
        .get_result::<Skill>(&mut establish_connection())
    {
        Ok(updated) => {
            let response = Response {
                body: ResponseBody::Skill(updated),
            };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!("Skill with id {} not found", skill_id_value)),
            };
            Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
