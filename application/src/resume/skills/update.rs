use diesel::prelude::*;
use domain::models::{Resume, Skill, UpdateSkill};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn update_skill(
    user_id_value: i32,
    skill_id_value: i32,
    payload: UpdateSkill,
) -> Result<Skill, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::skills;

    let existing: Skill = match skills::table
        .find(skill_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Skill with id {} not found",
                skill_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(skills::table.find(skill_id_value))
        .set(&payload)
        .get_result::<Skill>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Skill with id {} not found",
            skill_id_value
        ))),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
