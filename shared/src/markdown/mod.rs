use domain::models::ParsedResume;

mod dates;
mod export;
mod front_matter;
mod parse;

pub use dates::{format_markdown_date, parse_markdown_date};
pub use export::resume_to_markdown;
pub use front_matter::FrontMatter;
pub use parse::{markdown_to_resume, parse_resume_markdown};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownError {
    InvalidMarkdown(String),
}

/// A markdown document split into its optional front-matter and the parsed
/// resume body.
#[derive(Debug)]
pub struct ParsedMarkdown {
    pub front_matter: Option<FrontMatter>,
    pub resume: ParsedResume,
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::models::{DatePrecision, FullResume};

    pub(crate) fn resume_row() -> domain::models::Resume {
        domain::models::Resume {
            id: 7,
            name: "Jane Doe".to_string(),
            profile_image_url: None,
            location: None,
            email: "jane@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            created_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_by: Some(1),
            is_public: true,
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

    pub(crate) fn full_resume(resume: domain::models::Resume) -> FullResume {
        FullResume {
            resume,
            education: vec![],
            education_key_points: std::collections::HashMap::new(),
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: std::collections::HashMap::new(),
            portfolio_projects: vec![],
            portfolio_key_points: std::collections::HashMap::new(),
            portfolio_technologies: std::collections::HashMap::new(),
            languages: vec![],
            frameworks: std::collections::HashMap::new(),
        }
    }

    #[test]
    #[test]
    fn test_export_import_round_trip_front_matter() {
        let mut resume = resume_row();
        resume.id = 21;
        resume.base_resume_id = Some(7);
        resume.company_name = Some("Acme".to_string());

        let exported = resume_to_markdown(&full_resume(resume), true);
        let parsed = parse_resume_markdown(&exported).expect("round-trip parse ok");
        let fm = parsed.front_matter.expect("front-matter");
        assert_eq!(fm.resume_id, Some(serde_json::json!(21)));
        assert_eq!(fm.company_name, Some(serde_json::json!("Acme")));
        assert_eq!(parsed.resume.name, "Jane Doe");
    }

    pub(crate) fn education_row(active: bool) -> domain::models::Education {
        domain::models::Education {
            id: 11,
            resume_id: 7,
            education_stage: "Bachelor's".to_string(),
            institution_name: "University of ABC".to_string(),
            degree: None,
            start_date: NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 5, 1).unwrap()),
            description: None,
            display_order: Some(0),
            created_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            active,
            start_date_precision: "month".to_string(),
            end_date_precision: Some("month".to_string()),
        }
    }

    pub(crate) fn work_row(active: bool) -> domain::models::WorkExperience {
        domain::models::WorkExperience {
            id: 21,
            resume_id: 7,
            job_title: "Engineer".to_string(),
            company_name: Some("Tech Corp".to_string()),
            start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            end_date: None,
            description: None,
            display_order: Some(0),
            created_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            active,
            start_date_precision: "month".to_string(),
            end_date_precision: None,
        }
    }

    pub(crate) fn project_row(active: bool) -> domain::models::PortfolioProject {
        domain::models::PortfolioProject {
            id: 31,
            resume_id: 7,
            project_name: "My Project".to_string(),
            image_url: None,
            project_link: None,
            source_code_link: None,
            description: None,
            display_order: Some(0),
            created_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            active,
            video_url: None,
        }
    }

    pub(crate) fn ts() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    }

    pub(crate) fn active_fixture(active: bool) -> FullResume {
        let mut resume = full_resume(resume_row());
        resume.education.push(education_row(active));
        resume.education_key_points.insert(
            11,
            vec![domain::models::EducationKeyPoint {
                id: 111,
                education_id: 11,
                key_point: "Graduated".to_string(),
                display_order: None,
                created_at: ts(),
                active,
            }],
        );
        resume.work_experiences.push(work_row(active));
        resume.work_experience_key_points.insert(
            21,
            vec![domain::models::WorkExperienceKeyPoint {
                id: 211,
                work_experience_id: 21,
                key_point: "Did work".to_string(),
                display_order: None,
                created_at: ts(),
                active,
            }],
        );
        resume.portfolio_projects.push(project_row(active));
        resume.portfolio_key_points.insert(
            31,
            vec![domain::models::PortfolioKeyPoint {
                id: 311,
                portfolio_project_id: 31,
                key_point: "Built stuff".to_string(),
                display_order: None,
                created_at: ts(),
                active,
            }],
        );
        resume.portfolio_technologies.insert(
            31,
            vec![
                domain::models::PortfolioTechnology {
                    id: 411,
                    portfolio_project_id: 31,
                    technology_name: "Rust".to_string(),
                    display_order: None,
                    created_at: ts(),
                    active,
                },
                domain::models::PortfolioTechnology {
                    id: 412,
                    portfolio_project_id: 31,
                    technology_name: "Rocket".to_string(),
                    display_order: None,
                    created_at: ts(),
                    active: true,
                },
            ],
        );
        resume
    }

    #[test]
    #[test]
    fn test_active_marker_export_parse_round_trip() {
        let output = resume_to_markdown(&active_fixture(false), true);
        let parsed = markdown_to_resume(&output).expect("reparse ok");
        assert!(!parsed.education[0].active);
        assert!(!parsed.education_key_points[0].active);
        assert!(!parsed.work_experiences[0].active);
        assert!(!parsed.work_experience_key_points[0].active);
        assert!(!parsed.portfolio_projects[0].active);
        assert!(!parsed.portfolio_key_points[0].active);
        assert!(!parsed.portfolio_technologies[0].active);
        assert!(parsed.portfolio_technologies[1].active);
    }
}
