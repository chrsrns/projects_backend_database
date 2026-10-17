use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{PartialDate, Visibility};

/// Shape version of the stored snapshot document. A breaking change to the
/// snapshot bumps this; additive optional fields do not.
pub const SNAPSHOT_SCHEMA_VERSION: i32 = 1;

/// The restorable content of a resume: every column a restore writes back,
/// plus every child collection. Identity and lifecycle fields (`id`,
/// `created_at`, `updated_at`, `created_by`, `base_resume_id`) are deliberately
/// absent so a restore can never rewrite them.
///
/// Child collections are keyed by their stored child id, mirroring the shape
/// the aggregate is read in. A restore reinserts children with fresh ids, so
/// the keys are a snapshot of the ids at capture time, not a stable identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumeSnapshot {
    pub schema_version: i32,
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<String>,
    pub video: Option<String>,
    pub visibility: Visibility,
    pub company_name: Option<String>,
    pub role_title: Option<String>,
    #[schema(value_type = Option<PartialDate>)]
    pub target_date: Option<PartialDate>,
    pub target_date_precision: Option<String>,
    pub job_description: Option<String>,
    pub variant_label: Option<String>,
    pub show_variant_tag: bool,
    #[serde(default)]
    pub education: Vec<SnapshotEducation>,
    #[serde(default)]
    pub education_key_points: BTreeMap<i32, Vec<SnapshotEducationKeyPoint>>,
    #[serde(default)]
    pub skills: Vec<SnapshotSkill>,
    #[serde(default)]
    pub work_experiences: Vec<SnapshotWorkExperience>,
    #[serde(default)]
    pub work_experience_key_points: BTreeMap<i32, Vec<SnapshotWorkExperienceKeyPoint>>,
    #[serde(default)]
    pub portfolio_projects: Vec<SnapshotPortfolioProject>,
    #[serde(default)]
    pub portfolio_key_points: BTreeMap<i32, Vec<SnapshotPortfolioKeyPoint>>,
    #[serde(default)]
    pub portfolio_technologies: BTreeMap<i32, Vec<SnapshotPortfolioTechnology>>,
    #[serde(default)]
    pub languages: Vec<SnapshotLanguage>,
    #[serde(default)]
    pub frameworks: BTreeMap<i32, Vec<SnapshotFramework>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotEducation {
    pub id: i32,
    pub education_stage: String,
    pub institution_name: String,
    pub degree: Option<String>,
    #[schema(value_type = PartialDate)]
    pub start_date: PartialDate,
    #[schema(value_type = Option<PartialDate>)]
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotEducationKeyPoint {
    pub id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotSkill {
    pub id: i32,
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotWorkExperience {
    pub id: i32,
    pub job_title: String,
    pub company_name: Option<String>,
    #[schema(value_type = PartialDate)]
    pub start_date: PartialDate,
    #[schema(value_type = Option<PartialDate>)]
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotWorkExperienceKeyPoint {
    pub id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotPortfolioProject {
    pub id: i32,
    pub project_name: String,
    pub image_url: Option<String>,
    pub project_link: Option<String>,
    pub source_code_link: Option<String>,
    pub video_url: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotPortfolioKeyPoint {
    pub id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotPortfolioTechnology {
    pub id: i32,
    pub technology_name: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotLanguage {
    pub id: i32,
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct SnapshotFramework {
    pub id: i32,
    pub framework_name: String,
    pub display_order: Option<i32>,
}

/// A stored revision row, mapped in table column order.
#[derive(Queryable, Debug, Clone)]
pub struct ResumeRevisionRow {
    pub resume_id: Uuid,
    pub revision: i32,
    pub created_at: NaiveDateTime,
    pub created_by: i32,
    pub action: String,
    pub is_restore: bool,
    pub snapshot: serde_json::Value,
}

/// One timeline entry, without the snapshot body.
#[derive(Queryable, Serialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub struct ResumeRevisionSummary {
    pub revision: i32,
    pub created_at: NaiveDateTime,
    pub created_by: i32,
    pub action: String,
    pub is_restore: bool,
}

/// A timeline entry with the state it recorded.
#[derive(Serialize, ToSchema, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ResumeRevision {
    pub revision: i32,
    pub created_at: NaiveDateTime,
    pub created_by: i32,
    pub action: String,
    pub is_restore: bool,
    pub snapshot: ResumeSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RevisionChangeKind {
    Added,
    Removed,
    Changed,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct RevisionChange {
    pub path: String,
    pub kind: RevisionChangeKind,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumeRevisionDiff {
    pub revision: i32,
    pub changes: Vec<RevisionChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unified: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct RestoreRequest {
    pub expected_revision: i32,
}

impl ResumeSnapshot {
    /// Wraps the document in the `jsonb` shape the column stores.
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Reads a stored document. A field absent from an older snapshot reads as
    /// absent, which for every nullable column means `null`.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DatePrecision;
    use chrono::NaiveDate;

    fn sample_snapshot() -> ResumeSnapshot {
        let mut education_key_points = BTreeMap::new();
        education_key_points.insert(
            7,
            vec![SnapshotEducationKeyPoint {
                id: 11,
                key_point: "Graduated with honors".to_string(),
                display_order: Some(1),
                active: true,
            }],
        );

        let mut frameworks = BTreeMap::new();
        frameworks.insert(
            3,
            vec![SnapshotFramework {
                id: 4,
                framework_name: "Rocket".to_string(),
                display_order: None,
            }],
        );

        ResumeSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            name: "Snapshot Subject".to_string(),
            profile_image_url: None,
            location: Some("Here".to_string()),
            email: "snapshot@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: Some("Summary".to_string()),
            video: Some("https://example.com/v.mp4".to_string()),
            visibility: Visibility::Unlisted,
            company_name: Some("Acme".to_string()),
            role_title: None,
            target_date: Some(PartialDate {
                canonical: NaiveDate::from_ymd_opt(2026, 3, 1).expect("date"),
                precision: DatePrecision::Month,
            }),
            target_date_precision: Some("month".to_string()),
            job_description: None,
            variant_label: Some("Acme".to_string()),
            show_variant_tag: true,
            education: vec![SnapshotEducation {
                id: 7,
                education_stage: "Bachelor".to_string(),
                institution_name: "University".to_string(),
                degree: Some("BSc".to_string()),
                start_date: PartialDate {
                    canonical: NaiveDate::from_ymd_opt(2020, 1, 1).expect("date"),
                    precision: DatePrecision::Day,
                },
                end_date: None,
                description: None,
                display_order: Some(1),
                active: false,
            }],
            education_key_points,
            skills: vec![SnapshotSkill {
                id: 2,
                skill_name: "Rust".to_string(),
                confidence_percentage: 90,
                display_order: Some(1),
            }],
            work_experiences: Vec::new(),
            work_experience_key_points: BTreeMap::new(),
            portfolio_projects: Vec::new(),
            portfolio_key_points: BTreeMap::new(),
            portfolio_technologies: BTreeMap::new(),
            languages: vec![SnapshotLanguage {
                id: 3,
                language_name: "Rust".to_string(),
                display_order: None,
            }],
            frameworks,
        }
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let snapshot = sample_snapshot();
        let value = snapshot.to_json().expect("serialize");
        let restored = ResumeSnapshot::from_json(&value).expect("deserialize");

        assert_eq!(restored, snapshot);
    }

    #[test]
    fn snapshot_missing_a_nullable_field_deserializes() {
        let snapshot = sample_snapshot();
        let mut value = snapshot.to_json().expect("serialize");

        let object = value.as_object_mut().expect("object");
        object.remove("video");
        object.remove("executive_summary");

        let restored = ResumeSnapshot::from_json(&value).expect("missing fields read as absent");

        assert_eq!(restored.video, None);
        assert_eq!(restored.executive_summary, None);
        assert_eq!(restored.name, snapshot.name);
    }

    #[test]
    fn snapshot_missing_a_child_collection_deserializes_as_empty() {
        let snapshot = sample_snapshot();
        let mut value = snapshot.to_json().expect("serialize");

        value
            .as_object_mut()
            .expect("object")
            .remove("education_key_points");

        let restored = ResumeSnapshot::from_json(&value).expect("missing collection reads empty");

        assert!(restored.education_key_points.is_empty());
    }

    #[test]
    fn snapshot_serializes_child_maps_keyed_by_child_id() {
        let snapshot = sample_snapshot();
        let value = snapshot.to_json().expect("serialize");

        assert_eq!(
            value["education_key_points"]["7"][0]["key_point"].as_str(),
            Some("Graduated with honors")
        );
        assert_eq!(value["frameworks"]["3"][0]["id"].as_i64(), Some(4));
        assert_eq!(
            value["education"][0]["start_date"].as_str(),
            Some("2020-01-01")
        );
        assert_eq!(value["visibility"].as_str(), Some("unlisted"));
    }
}
