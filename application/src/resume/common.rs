use chrono::NaiveDate;
use diesel::Connection;
use diesel::prelude::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::methods::LimitDsl;
use domain::models::{PartialDate, Resume, Visibility};
use infrastructure::{DbConnection, establish_connection, run_in_transaction};

use crate::error::ApplicationError;
use crate::realtime::{ResumeChangePublisher, ResumeChangedAction};
use crate::resume::revision::capture::capture_for_action;
use uuid::Uuid;

fn entity_not_found(entity: &str, id: impl std::fmt::Display) -> ApplicationError {
    ApplicationError::NotFound(format!("{entity} with id {id} not found"))
}

/// Runs `query.first` on a freshly checked-out pooled connection and maps a
/// miss to a uniform "{entity} with id {id} not found" 404 body.
pub fn first_or_404<'q, Q, T>(
    entity: &str,
    id: impl std::fmt::Display,
    query: Q,
) -> Result<T, ApplicationError>
where
    Q: LimitDsl + diesel::RunQueryDsl<DbConnection>,
    <Q as LimitDsl>::Output: LoadQuery<'q, DbConnection, T>,
{
    query
        .first(&mut establish_connection())
        .map_err(|err| match err {
            diesel::result::Error::NotFound => entity_not_found(entity, id),
            err => app_err_from_diesel_err(err),
        })
}

/// Loads the resume by id (404 when absent) then requires `user_id` to own
/// it (403 when foreign), in that order.
pub fn require_owned_resume(user_id: i32, resume_id: Uuid) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;

    let resume = first_or_404("Resume", resume_id, resumes::table.find(resume_id))?;
    require_owner(&resume, user_id)?;
    Ok(resume)
}

/// Runs `op` and its revision capture in one transaction on a pooled
/// connection. The capture holds a lock on the resume row while it reads the
/// pre-state and picks the next revision number, so concurrent edits of one
/// resume serialize instead of recording a stale before-image. On success
/// publishes `resume_changed` for `resume_id`; a `NotFound` miss produces the
/// uniform "{entity} with id {id} not found" body; all other errors map through
/// `app_err_from_diesel_err`.
pub fn run_published<T>(
    entity: &str,
    id: impl std::fmt::Display,
    resume_id: Uuid,
    action: ResumeChangedAction,
    publisher: &dyn ResumeChangePublisher,
    op: impl FnOnce(&mut DbConnection) -> diesel::result::QueryResult<T>,
) -> Result<T, ApplicationError> {
    let mut conn = establish_connection();

    match run_in_transaction(&mut conn, |conn| {
        capture_for_action(conn, resume_id, &action)?;
        op(conn)
    }) {
        Ok(item) => {
            publisher.resume_changed(resume_id, action);
            Ok(item)
        }
        Err(diesel::result::Error::NotFound) => Err(entity_not_found(entity, id)),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

/// Runs a delete `op` and its revision capture in one transaction on a pooled
/// connection. A miss (zero rows deleted) rolls the transaction back — the
/// capture goes with it — and returns the uniform "{entity} with id {id} not
/// found" body; otherwise publishes `resume_changed` with `action` for
/// `resume_id`.
pub fn run_published_delete(
    entity: &str,
    id: impl std::fmt::Display,
    resume_id: Uuid,
    action: ResumeChangedAction,
    publisher: &dyn ResumeChangePublisher,
    op: impl FnOnce(&mut DbConnection) -> diesel::result::QueryResult<usize>,
) -> Result<(), ApplicationError> {
    let mut conn = establish_connection();

    match conn.transaction::<usize, DeleteMissOr, _>(|conn| {
        capture_for_action(conn, resume_id, &action)?;
        match op(conn)? {
            0 => Err(DeleteMissOr::Miss),
            deleted => Ok(deleted),
        }
    }) {
        Ok(_) => {
            publisher.resume_changed(resume_id, action);
            Ok(())
        }
        Err(DeleteMissOr::Miss) | Err(DeleteMissOr::Db(diesel::result::Error::NotFound)) => {
            Err(entity_not_found(entity, id))
        }
        Err(DeleteMissOr::Db(err)) => Err(app_err_from_diesel_err(err)),
    }
}

/// Transaction error for `run_published_delete`: a zero-row delete is not a
/// database failure, but returning it as an error is what rolls the capture
/// back with the rest of the transaction.
enum DeleteMissOr {
    Miss,
    Db(diesel::result::Error),
}

impl From<diesel::result::Error> for DeleteMissOr {
    fn from(err: diesel::result::Error) -> Self {
        DeleteMissOr::Db(err)
    }
}

pub fn find_resume(resume_id: Uuid) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;

    first_or_404("Resume", resume_id, resumes::table.find(resume_id))
}

pub fn find_accessible_resume(
    resume_id_value: Uuid,
    user_id_value: Option<i32>,
) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::visibility
                .ne(Visibility::Private.as_str())
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::visibility.ne(Visibility::Private.as_str())),
    };

    first_or_404("Resume", resume_id_value, resume_query)
}

pub fn require_owner(resume: &Resume, user_id: i32) -> Result<(), ApplicationError> {
    if resume.created_by == user_id {
        Ok(())
    } else {
        Err(ApplicationError::Forbidden)
    }
}

pub fn viewer_is_owner(resume: &Resume, viewer: Option<i32>) -> bool {
    Some(resume.created_by) == viewer
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

/// Same tri-state normalization as `validate_optional_text_update` with the
/// 500-character URL limit shared by every URL-class patch field.
pub fn validate_optional_url_update(
    value: Option<Option<String>>,
    field_name: &str,
) -> Result<Option<Option<String>>, ApplicationError> {
    validate_optional_text_update(value, field_name, 500)
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

/// Normalizes a nullable text field on update.
///
/// Absent and an empty string both mean "leave the stored value alone", an
/// explicit `null` clears it, and whitespace-only text normalizes to `null`
/// so the column never holds a blank string.
pub fn validate_optional_text_update(
    value: Option<Option<String>>,
    field_name: &str,
    max_length: usize,
) -> Result<Option<Option<String>>, ApplicationError> {
    match value {
        None => Ok(None),
        Some(None) => Ok(Some(None)),
        Some(Some(text)) if text.is_empty() => Ok(None),
        Some(Some(text)) => Ok(Some(validate_optional_text(
            Some(text),
            field_name,
            max_length,
        )?)),
    }
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
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            _,
        ) => ApplicationError::Conflict("Foreign key violation".to_string()),
        diesel::result::Error::NotFound => ApplicationError::NotFound("Not found".to_string()),
        _ => ApplicationError::Internal(format!("Database error - {}", err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn resume_with_owner(owner: i32) -> Resume {
        Resume {
            id: Uuid::nil(),
            name: "n".to_string(),
            profile_image_url: None,
            location: None,
            email: "e@x.com".to_string(),
            github_url: None,
            mobile_number: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            created_by: owner,
            visibility: "private".to_string(),
            executive_summary: None,
            video: None,
            base_resume_id: None,
            company_name: None,
            role_title: None,
            target_date: None,
            target_date_precision: None,
            job_description: None,
            variant_label: None,
            show_variant_tag: true,
        }
    }

    #[test]
    fn test_entity_not_found_formats_body() {
        assert_eq!(
            entity_not_found("Education", 42),
            ApplicationError::NotFound("Education with id 42 not found".to_string())
        );
    }

    #[test]
    fn test_require_owner_accepts_owner() {
        assert!(require_owner(&resume_with_owner(7), 7).is_ok());
    }

    #[test]
    fn test_require_owner_rejects_non_owner() {
        assert_eq!(
            require_owner(&resume_with_owner(7), 8),
            Err(ApplicationError::Forbidden)
        );
    }

    #[test]
    fn test_viewer_is_owner_matching() {
        assert!(viewer_is_owner(&resume_with_owner(7), Some(7)));
    }

    #[test]
    fn test_viewer_is_owner_mismatch() {
        assert!(!viewer_is_owner(&resume_with_owner(7), Some(8)));
    }

    #[test]
    fn test_viewer_is_owner_anonymous() {
        assert!(!viewer_is_owner(&resume_with_owner(7), None));
    }

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
    fn test_validate_optional_url_update_absent_means_no_change() {
        assert_eq!(
            validate_optional_url_update(None, "Video URL").unwrap(),
            None
        );
    }

    #[test]
    fn test_validate_optional_url_update_null_stores_null() {
        assert_eq!(
            validate_optional_url_update(Some(None), "Video URL").unwrap(),
            Some(None)
        );
    }

    #[test]
    fn test_validate_optional_url_update_empty_string_means_no_change() {
        assert_eq!(
            validate_optional_url_update(Some(Some("".to_string())), "Video URL").unwrap(),
            None
        );
    }

    #[test]
    fn test_validate_optional_url_update_blank_text_stores_null() {
        assert_eq!(
            validate_optional_url_update(Some(Some("   ".to_string())), "Video URL").unwrap(),
            Some(None)
        );
    }

    #[test]
    fn test_validate_optional_url_update_rejects_over_500() {
        let long = "x".repeat(501);
        match validate_optional_url_update(Some(Some(long)), "Video URL") {
            Err(ApplicationError::BadRequest(msg)) => {
                assert_eq!(msg, "Video URL must be at most 500 characters");
            }
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_optional_url_update_accepts_value() {
        let url = "https://example.com/p.png".to_string();
        assert_eq!(
            validate_optional_url_update(Some(Some(url.clone())), "Image URL").unwrap(),
            Some(Some(url))
        );
    }

    #[test]
    fn test_foreign_key_violation_maps_to_conflict() {
        let error = diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            Box::new("foreign key violation".to_string()),
        );

        assert_eq!(
            app_err_from_diesel_err(error),
            ApplicationError::Conflict("Foreign key violation".to_string())
        );
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
