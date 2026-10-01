use chrono::NaiveDate;
use diesel::prelude::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::models::{PartialDate, Resume};
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

pub fn validate_optional_text(
    value: Option<String>,
    field_name: &str,
    max_length: usize,
) -> Result<Option<String>, ApplicationError> {
    match value {
        None => Ok(None),
        Some(value) if value.trim().is_empty() => Ok(None),
        Some(value) if value.chars().count() > max_length => Err(ApplicationError::BadRequest(
            format!("{} must be at most {} characters", field_name, max_length),
        )),
        Some(value) => Ok(Some(value)),
    }
}

pub fn validate_optional_url(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, ApplicationError> {
    validate_optional_text(value, field_name, 500)
}

pub fn validate_video(video: Option<String>) -> Result<Option<String>, ApplicationError> {
    validate_optional_url(video, "Video")
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariantMetadata {
    pub company_name: Option<String>,
    pub role_title: Option<String>,
    pub variant_label: Option<String>,
    pub job_description: Option<String>,
}

pub fn validate_variant_metadata(
    company_name: Option<String>,
    role_title: Option<String>,
    variant_label: Option<String>,
    job_description: Option<String>,
) -> Result<VariantMetadata, ApplicationError> {
    Ok(VariantMetadata {
        company_name: validate_optional_text(company_name, "Company name", 255)?,
        role_title: validate_optional_text(role_title, "Role title", 255)?,
        variant_label: validate_optional_text(variant_label, "Variant label", 255)?,
        job_description: validate_optional_text(job_description, "Job description", 20_000)?,
    })
}

pub fn partial_date_to_columns(value: PartialDate) -> (NaiveDate, String) {
    (value.canonical_start_date(), value.precision.to_string())
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

    #[test]
    fn test_validate_variant_metadata_normalizes_blank_to_none() {
        let metadata = validate_variant_metadata(
            Some("   ".to_string()),
            Some("\t".to_string()),
            Some("\n".to_string()),
            Some("  ".to_string()),
        )
        .unwrap();

        assert_eq!(
            metadata,
            VariantMetadata {
                company_name: None,
                role_title: None,
                variant_label: None,
                job_description: None,
            }
        );
    }

    #[test]
    fn test_validate_variant_metadata_rejects_company_name_over_255() {
        match validate_variant_metadata(Some("x".repeat(256)), None, None, None) {
            Err(ApplicationError::BadRequest(msg)) => {
                assert_eq!(msg, "Company name must be at most 255 characters");
            }
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_variant_metadata_rejects_role_title_over_255() {
        match validate_variant_metadata(None, Some("x".repeat(256)), None, None) {
            Err(ApplicationError::BadRequest(msg)) => {
                assert_eq!(msg, "Role title must be at most 255 characters");
            }
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_variant_metadata_rejects_variant_label_over_255() {
        match validate_variant_metadata(None, None, Some("x".repeat(256)), None) {
            Err(ApplicationError::BadRequest(msg)) => {
                assert_eq!(msg, "Variant label must be at most 255 characters");
            }
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_variant_metadata_rejects_job_description_over_20000() {
        match validate_variant_metadata(None, None, None, Some("x".repeat(20_001))) {
            Err(ApplicationError::BadRequest(msg)) => {
                assert_eq!(msg, "Job description must be at most 20000 characters");
            }
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_variant_metadata_accepts_boundary_lengths() {
        let metadata = validate_variant_metadata(
            Some("x".repeat(255)),
            Some("x".repeat(255)),
            Some("x".repeat(255)),
            Some("x".repeat(20_000)),
        )
        .unwrap();

        assert_eq!(metadata.company_name.unwrap().chars().count(), 255);
        assert_eq!(metadata.job_description.unwrap().chars().count(), 20_000);
    }
}
