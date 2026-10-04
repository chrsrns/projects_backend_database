use chrono::NaiveDateTime;
use rocket::serde::Serialize;
use std::str::FromStr;
use utoipa::ToSchema;

use super::{DatePrecision, PartialDate, Resume};

/// Viewer-aware, flat resume payload.
///
/// The stored row carries variant metadata for every resume, but what a
/// client may see depends on who is asking: whether a variant's base is
/// reachable, and whether the viewer owns the row. Those three values are
/// computed per request here rather than stored, so the database never
/// holds a viewer-specific answer.
#[derive(Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumeView {
    pub id: i32,
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: Option<i32>,
    pub is_public: bool,
    pub executive_summary: Option<String>,
    #[schema(max_length = 500)]
    pub video: Option<String>,
    pub is_variant: bool,
    pub base_resume_id: Option<i32>,
    pub show_variant_tag: Option<bool>,
    pub company_name: Option<String>,
    pub role_title: Option<String>,
    #[schema(value_type = Option<PartialDate>)]
    pub target_date: Option<PartialDate>,
    pub target_date_precision: Option<String>,
    pub job_description: Option<String>,
    pub variant_label: Option<String>,
}

impl ResumeView {
    pub fn from_resume(resume: Resume, base_accessible: bool, is_owner: bool) -> Self {
        let target_date = resume.target_date.map(|canonical| PartialDate {
            canonical,
            precision: resume
                .target_date_precision
                .as_deref()
                .and_then(|value| DatePrecision::from_str(value).ok())
                .unwrap_or(DatePrecision::Day),
        });

        ResumeView {
            is_variant: resume.base_resume_id.is_some()
                && (base_accessible || resume.show_variant_tag),
            base_resume_id: if base_accessible {
                resume.base_resume_id
            } else {
                None
            },
            show_variant_tag: if is_owner {
                Some(resume.show_variant_tag)
            } else {
                None
            },
            target_date,
            id: resume.id,
            name: resume.name,
            profile_image_url: resume.profile_image_url,
            location: resume.location,
            email: resume.email,
            github_url: resume.github_url,
            mobile_number: resume.mobile_number,
            created_at: resume.created_at,
            updated_at: resume.updated_at,
            created_by: resume.created_by,
            is_public: resume.is_public,
            executive_summary: resume.executive_summary,
            video: resume.video,
            company_name: resume.company_name,
            role_title: resume.role_title,
            target_date_precision: resume.target_date_precision,
            job_description: resume.job_description,
            variant_label: resume.variant_label,
        }
    }
}
