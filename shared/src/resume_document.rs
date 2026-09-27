use std::collections::BTreeMap;

use domain::models::PartialDate;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const RESUME_DOCUMENT_SCHEMA_VERSION: i32 = 1;
pub const RESUME_DOCUMENT_GENERATOR: &str = "projects_backend_database";

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumeDocumentEnvelope {
    pub schema_version: i32,
    pub generator: String,
    pub document: ResumeDocument,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumeDocument {
    pub resume: DocumentResume,
    pub education: Vec<DocumentEducation>,
    pub education_key_points: BTreeMap<i32, Vec<DocumentEducationKeyPoint>>,
    pub skills: Vec<DocumentSkill>,
    pub work_experiences: Vec<DocumentWorkExperience>,
    pub work_experience_key_points: BTreeMap<i32, Vec<DocumentWorkExperienceKeyPoint>>,
    pub portfolio_projects: Vec<DocumentPortfolioProject>,
    pub portfolio_key_points: BTreeMap<i32, Vec<DocumentPortfolioKeyPoint>>,
    pub portfolio_technologies: BTreeMap<i32, Vec<DocumentPortfolioTechnology>>,
    pub languages: Vec<DocumentLanguage>,
    pub frameworks: BTreeMap<i32, Vec<DocumentFramework>>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentResume {
    pub id: i32,
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<String>,
    pub is_public: bool,
    pub video: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentEducation {
    pub id: i32,
    pub resume_id: i32,
    pub education_stage: String,
    pub institution_name: String,
    pub degree: Option<String>,
    pub start_date: PartialDate,
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentEducationKeyPoint {
    pub id: i32,
    pub education_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentSkill {
    pub id: i32,
    pub resume_id: i32,
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentWorkExperience {
    pub id: i32,
    pub resume_id: i32,
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: PartialDate,
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentWorkExperienceKeyPoint {
    pub id: i32,
    pub work_experience_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentPortfolioProject {
    pub id: i32,
    pub resume_id: i32,
    pub project_name: String,
    pub image_url: Option<String>,
    pub project_link: Option<String>,
    pub source_code_link: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
    pub video_url: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentPortfolioKeyPoint {
    pub id: i32,
    pub portfolio_project_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentPortfolioTechnology {
    pub id: i32,
    pub portfolio_project_id: i32,
    pub technology_name: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentLanguage {
    pub id: i32,
    pub resume_id: i32,
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct DocumentFramework {
    pub id: i32,
    pub language_id: i32,
    pub framework_name: String,
    pub display_order: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct MarkdownValidationReport {
    pub valid: bool,
    pub errors: Vec<MarkdownValidationError>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct MarkdownValidationError {
    pub section: Option<String>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::DatePrecision;
    use rocket::serde::json::serde_json;

    fn year_date(year: i32) -> PartialDate {
        PartialDate {
            canonical: chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            precision: DatePrecision::Year,
        }
    }

    #[test]
    fn education_serializes_partial_date_as_iso_string() {
        let education = DocumentEducation {
            id: 1,
            resume_id: 0,
            education_stage: "Bachelor's".to_string(),
            institution_name: "University".to_string(),
            degree: None,
            start_date: year_date(2020),
            end_date: None,
            description: None,
            display_order: None,
            active: true,
        };
        let json = serde_json::to_value(&education).unwrap();
        assert_eq!(json["start_date"], "2020");
        assert_eq!(json["end_date"], serde_json::Value::Null);
        assert_eq!(json["active"], true);
        assert!(json.get("created_at").is_none());
        assert!(json.get("updated_at").is_none());
        assert!(json.get("created_by").is_none());
    }

    #[test]
    fn child_map_keys_serialize_as_json_strings() {
        let mut map = BTreeMap::new();
        map.insert(
            2,
            vec![DocumentFramework {
                id: 1,
                language_id: 2,
                framework_name: "Rocket".to_string(),
                display_order: None,
            }],
        );
        let json = serde_json::to_value(&map).unwrap();
        assert!(json.get("2").is_some());
    }

    #[test]
    fn validation_report_serializes_nullable_section() {
        let report = MarkdownValidationReport {
            valid: false,
            errors: vec![MarkdownValidationError {
                section: None,
                message: "Missing required field: Email".to_string(),
            }],
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["valid"], false);
        assert_eq!(json["errors"][0]["section"], serde_json::Value::Null);
        assert_eq!(
            json["errors"][0]["message"],
            "Missing required field: Email"
        );
    }

    #[test]
    fn envelope_round_trips_through_json() {
        let document = ResumeDocument {
            resume: DocumentResume {
                id: 0,
                name: "Jane".to_string(),
                profile_image_url: None,
                location: None,
                email: "jane@example.com".to_string(),
                github_url: None,
                mobile_number: None,
                executive_summary: None,
                is_public: true,
                video: None,
            },
            education: vec![],
            education_key_points: BTreeMap::new(),
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: BTreeMap::new(),
            portfolio_projects: vec![],
            portfolio_key_points: BTreeMap::new(),
            portfolio_technologies: BTreeMap::new(),
            languages: vec![],
            frameworks: BTreeMap::new(),
        };
        let envelope = ResumeDocumentEnvelope {
            schema_version: RESUME_DOCUMENT_SCHEMA_VERSION,
            generator: RESUME_DOCUMENT_GENERATOR.to_string(),
            document,
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let back: ResumeDocumentEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, 1);
        assert_eq!(back.generator, "projects_backend_database");
        assert_eq!(back.document.resume.id, 0);
        assert_eq!(back.document.resume.email, "jane@example.com");
    }
}
