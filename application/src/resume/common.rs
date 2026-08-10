use diesel::prelude::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::models::Resume;
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn find_resume(resume_id: i32) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;
    use infrastructure::establish_connection;

    let existing: Resume = match resumes::table
        .find(resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };
    Ok(existing)
}

pub fn find_accessible_resume(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Resume, ApplicationError> {
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

    match resume_query.first(&mut establish_connection()) {
        Ok(r) => Ok(r),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Resume with id {} not found",
            resume_id_value
        ))),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn validate_executive_summary(
    summary: Option<String>,
) -> Result<Option<String>, ApplicationError> {
    match summary {
        None => Ok(None),
        Some(summary) if summary.trim().is_empty() => Ok(None),
        Some(summary) if summary.chars().count() > 5000 => Err(ApplicationError::BadRequest(
            "Executive summary must be at most 5000 characters".to_string(),
        )),
        Some(summary) => Ok(Some(summary)),
    }
}

pub fn validate_optional_url(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, ApplicationError> {
    match value {
        None => Ok(None),
        Some(value) if value.trim().is_empty() => Ok(None),
        Some(value) if value.chars().count() > 500 => Err(ApplicationError::BadRequest(format!(
            "{} must be at most 500 characters",
            field_name
        ))),
        Some(value) => Ok(Some(value)),
    }
}

pub fn validate_video(video: Option<String>) -> Result<Option<String>, ApplicationError> {
    validate_optional_url(video, "Video")
}

pub fn app_err_from_diesel_err(err: diesel::result::Error) -> ApplicationError {
    match err {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => ApplicationError::Conflict("Unique violation".to_string()),
        diesel::result::Error::NotFound => ApplicationError::NotFound("Not found".to_string()),
        _ => ApplicationError::Internal(format!("Database error - {}", err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_optional_url_accepts_valid_url() {
        let url = "https://example.com/video.mp4".to_string();
        assert_eq!(
            validate_optional_url(Some(url.clone()), "Video URL").unwrap(),
            Some(url)
        );
    }

    #[test]
    fn test_validate_optional_url_normalizes_blank_to_none() {
        assert_eq!(
            validate_optional_url(Some("   \t\n  ".to_string()), "Video URL").unwrap(),
            None
        );
    }

    #[test]
    fn test_validate_optional_url_rejects_too_long() {
        let long = "x".repeat(501);
        match validate_optional_url(Some(long), "Video URL") {
            Err(ApplicationError::BadRequest(msg)) => {
                assert!(msg.contains("Video URL must be at most 500 characters"));
            }
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_optional_url_passes_none() {
        assert_eq!(validate_optional_url(None, "Video URL").unwrap(), None);
    }
}
