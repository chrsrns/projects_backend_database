use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, NewEducation, NewEducationKeyPoint, NewEducationKeyPointRequest,
    NewEducationRequest, Resume,
};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use shared::response_models::{Response, ResponseBody};

pub fn create_education(
    user_id_value: i32,
    resume_id_value: i32,
    payload: Json<NewEducationRequest>,
) -> Result<Created<String>, Custom<String>> {
    use domain::schema::education;
    use domain::schema::resumes;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Resume with id {} not found",
                    resume_id_value
                )),
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
    let new_education = NewEducation {
        resume_id: resume_id_value,
        education_stage: payload.education_stage,
        institution_name: payload.institution_name,
        degree: payload.degree,
        start_date: payload.start_date,
        end_date: payload.end_date,
        description: payload.description,
        display_order: payload.display_order,
    };

    match diesel::insert_into(education::table)
        .values(&new_education)
        .get_result::<Education>(&mut establish_connection())
    {
        Ok(item) => {
            let response = Response {
                body: ResponseBody::Education(item),
            };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}

pub fn create_education_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    education_id_value: i32,
    payload: Json<NewEducationKeyPointRequest>,
) -> Result<Created<String>, Custom<String>> {
    use domain::schema::education::dsl as education_dsl;
    use domain::schema::education_key_points;
    use domain::schema::resumes;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Resume with id {} not found",
                    resume_id_value
                )),
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

    let _education: Education = match education_dsl::education
        .filter(education_dsl::id.eq(education_id_value))
        .filter(education_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(e) => e,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Education not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let payload = payload.into_inner();
    let new_kp = NewEducationKeyPoint {
        education_id: education_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
    };

    match diesel::insert_into(education_key_points::table)
        .values(&new_kp)
        .get_result::<EducationKeyPoint>(&mut establish_connection())
    {
        Ok(item) => {
            let response = Response {
                body: ResponseBody::EducationKeyPoint(item),
            };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
