use diesel::prelude::*;
use domain::models::{Resume, ResumeView, UpdateResume, UpdateResumeChangeset};
use domain::schema::resumes;
use infrastructure::establish_connection;

use crate::error::ApplicationError;
use crate::realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType};
use crate::resume::common::{
    app_err_from_diesel_err, find_resume, partial_date_to_columns, require_owner,
    validate_executive_summary, validate_optional_text_update, validate_optional_url_update,
};

pub fn update_resume(
    user_id_value: i32,
    resume_id: i32,
    mut resume: UpdateResume,
    publisher: &dyn ResumeChangePublisher,
) -> Result<ResumeView, ApplicationError> {
    let existing = find_resume(resume_id)?;

    require_owner(&existing, user_id_value)?;

    if existing.base_resume_id.is_some() && resume.email.is_some() {
        return Err(ApplicationError::BadRequest(
            "A variant's email cannot be changed".to_string(),
        ));
    }

    let writes_variant_metadata = resume.company_name.is_some()
        || resume.role_title.is_some()
        || resume.variant_label.is_some()
        || resume.job_description.is_some()
        || resume.target_date.is_some()
        || resume.show_variant_tag.is_some();

    if existing.base_resume_id.is_none() && writes_variant_metadata {
        return Err(ApplicationError::BadRequest(
            "Variant metadata can only be set on a variant resume".to_string(),
        ));
    }

    resume.company_name = validate_optional_text_update(resume.company_name, "Company name", 255)?;
    resume.role_title = validate_optional_text_update(resume.role_title, "Role title", 255)?;
    resume.variant_label =
        validate_optional_text_update(resume.variant_label, "Variant label", 255)?;
    resume.job_description =
        validate_optional_text_update(resume.job_description, "Job description", 20_000)?;

    resume.executive_summary = resume
        .executive_summary
        .map(validate_executive_summary)
        .transpose()?;

    resume.video = validate_optional_url_update(resume.video, "Video")?;

    if resume.name.is_none()
        && resume.profile_image_url.is_none()
        && resume.location.is_none()
        && resume.email.is_none()
        && resume.github_url.is_none()
        && resume.mobile_number.is_none()
        && resume.executive_summary.is_none()
        && resume.video.is_none()
        && resume.is_public.is_none()
        && resume.company_name.is_none()
        && resume.role_title.is_none()
        && resume.variant_label.is_none()
        && resume.job_description.is_none()
        && resume.target_date.is_none()
        && resume.show_variant_tag.is_none()
    {
        // Only the owner reaches this point, and a variant's base is always
        // owned by the same user, so the base is reachable.
        publisher.resume_changed(
            resume_id,
            ResumeChangedAction::Updated(SectionType::PersonalInfo),
        );
        return Ok(ResumeView::from_resume(existing, true, true));
    }

    let changeset = UpdateResumeChangeset {
        name: resume.name,
        profile_image_url: resume.profile_image_url,
        location: resume.location,
        email: resume.email,
        github_url: resume.github_url,
        mobile_number: resume.mobile_number,
        executive_summary: resume.executive_summary,
        video: resume.video,
        is_public: resume.is_public,
        company_name: resume.company_name,
        role_title: resume.role_title,
        variant_label: resume.variant_label,
        job_description: resume.job_description,
        target_date: resume
            .target_date
            .map(|value| value.map(|date| partial_date_to_columns(date).0)),
        target_date_precision: resume
            .target_date
            .map(|value| value.map(|date| partial_date_to_columns(date).1)),
        show_variant_tag: resume.show_variant_tag,
    };

    match diesel::update(resumes::table.find(resume_id))
        .set(&changeset)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(updated_resume) => {
            publisher.resume_changed(
                updated_resume.id,
                ResumeChangedAction::Updated(SectionType::PersonalInfo),
            );
            Ok(ResumeView::from_resume(updated_resume, true, true))
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
