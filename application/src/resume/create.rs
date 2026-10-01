use diesel::prelude::*;
use domain::models::{NewResume, NewResumeRequest, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, validate_executive_summary, validate_video},
};

pub fn create_resume(
    user_id_value: i32,
    resume: NewResumeRequest,
) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;

    let new_resume = NewResume {
        name: resume.name,
        profile_image_url: resume.profile_image_url,
        location: resume.location,
        email: resume.email,
        github_url: resume.github_url,
        mobile_number: resume.mobile_number,
        executive_summary: validate_executive_summary(resume.executive_summary)?,
        video: validate_video(resume.video)?,
        created_by: Some(user_id_value),
        is_public: resume.is_public.unwrap_or(false),
        base_resume_id: None,
        company_name: None,
        role_title: None,
        target_date: None,
        target_date_precision: None,
        job_description: None,
        variant_label: None,
        show_variant_tag: true,
    };

    match diesel::insert_into(resumes::table)
        .values(&new_resume)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(resume) => Ok(resume),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
