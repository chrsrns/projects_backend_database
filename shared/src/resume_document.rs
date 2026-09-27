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

impl From<domain::models::ParsedResume> for ResumeDocument {
    fn from(parsed: domain::models::ParsedResume) -> Self {
        let education: Vec<DocumentEducation> = parsed
            .education
            .iter()
            .enumerate()
            .map(|(index, edu)| DocumentEducation {
                id: index as i32 + 1,
                resume_id: 0,
                education_stage: edu.education_stage.clone(),
                institution_name: edu.institution_name.clone(),
                degree: edu.degree.clone(),
                start_date: edu.start_date,
                end_date: edu.end_date,
                description: edu.description.clone(),
                display_order: edu.display_order,
                active: true,
            })
            .collect();

        let mut education_key_points: BTreeMap<i32, Vec<DocumentEducationKeyPoint>> =
            BTreeMap::new();
        let mut next_education_kp_id: i32 = 1;
        for kp in parsed.education_key_points.iter() {
            let parent_id = kp.education_index as i32 + 1;
            if kp.education_index >= education.len() {
                continue;
            }
            let id = next_education_kp_id;
            next_education_kp_id += 1;
            education_key_points
                .entry(parent_id)
                .or_default()
                .push(DocumentEducationKeyPoint {
                    id,
                    education_id: parent_id,
                    key_point: kp.key_point.clone(),
                    display_order: None,
                    active: true,
                });
        }

        let skills: Vec<DocumentSkill> = parsed
            .skills
            .iter()
            .enumerate()
            .map(|(index, skill)| DocumentSkill {
                id: index as i32 + 1,
                resume_id: 0,
                skill_name: skill.skill_name.clone(),
                confidence_percentage: skill.confidence_percentage,
                display_order: skill.display_order,
            })
            .collect();

        let work_experiences: Vec<DocumentWorkExperience> = parsed
            .work_experiences
            .iter()
            .enumerate()
            .map(|(index, work)| DocumentWorkExperience {
                id: index as i32 + 1,
                resume_id: 0,
                job_title: work.job_title.clone(),
                company_name: work.company_name.clone(),
                start_date: work.start_date,
                end_date: work.end_date,
                description: work.description.clone(),
                display_order: work.display_order,
                active: true,
            })
            .collect();

        let mut work_experience_key_points: BTreeMap<i32, Vec<DocumentWorkExperienceKeyPoint>> =
            BTreeMap::new();
        let mut next_work_kp_id: i32 = 1;
        for kp in parsed.work_experience_key_points.iter() {
            let parent_id = kp.work_experience_index as i32 + 1;
            if kp.work_experience_index >= work_experiences.len() {
                continue;
            }
            let id = next_work_kp_id;
            next_work_kp_id += 1;
            work_experience_key_points
                .entry(parent_id)
                .or_default()
                .push(DocumentWorkExperienceKeyPoint {
                    id,
                    work_experience_id: parent_id,
                    key_point: kp.key_point.clone(),
                    display_order: None,
                    active: true,
                });
        }

        let portfolio_projects: Vec<DocumentPortfolioProject> = parsed
            .portfolio_projects
            .iter()
            .enumerate()
            .map(|(index, project)| DocumentPortfolioProject {
                id: index as i32 + 1,
                resume_id: 0,
                project_name: project.project_name.clone(),
                image_url: project.image_url.clone(),
                project_link: project.project_link.clone(),
                source_code_link: project.source_code_link.clone(),
                description: project.description.clone(),
                display_order: project.display_order,
                active: true,
                video_url: project.video_url.clone(),
            })
            .collect();

        let mut portfolio_key_points: BTreeMap<i32, Vec<DocumentPortfolioKeyPoint>> =
            BTreeMap::new();
        let mut next_portfolio_kp_id: i32 = 1;
        for kp in parsed.portfolio_key_points.iter() {
            let parent_id = kp.portfolio_project_index as i32 + 1;
            if kp.portfolio_project_index >= portfolio_projects.len() {
                continue;
            }
            let id = next_portfolio_kp_id;
            next_portfolio_kp_id += 1;
            portfolio_key_points
                .entry(parent_id)
                .or_default()
                .push(DocumentPortfolioKeyPoint {
                    id,
                    portfolio_project_id: parent_id,
                    key_point: kp.key_point.clone(),
                    display_order: None,
                    active: true,
                });
        }

        let mut portfolio_technologies: BTreeMap<i32, Vec<DocumentPortfolioTechnology>> =
            BTreeMap::new();
        let mut next_technology_id: i32 = 1;
        for tech in parsed.portfolio_technologies.iter() {
            let parent_id = tech.portfolio_project_index as i32 + 1;
            if tech.portfolio_project_index >= portfolio_projects.len() {
                continue;
            }
            let id = next_technology_id;
            next_technology_id += 1;
            portfolio_technologies.entry(parent_id).or_default().push(
                DocumentPortfolioTechnology {
                    id,
                    portfolio_project_id: parent_id,
                    technology_name: tech.technology_name.clone(),
                    display_order: None,
                    active: true,
                },
            );
        }

        let languages: Vec<DocumentLanguage> = parsed
            .languages
            .iter()
            .enumerate()
            .map(|(index, language)| DocumentLanguage {
                id: index as i32 + 1,
                resume_id: 0,
                language_name: language.language_name.clone(),
                display_order: language.display_order,
            })
            .collect();

        let mut frameworks: BTreeMap<i32, Vec<DocumentFramework>> = BTreeMap::new();
        let mut next_framework_id: i32 = 1;
        for framework in parsed.frameworks.iter() {
            let parent_id = framework.language_index as i32 + 1;
            if framework.language_index >= languages.len() {
                continue;
            }
            let id = next_framework_id;
            next_framework_id += 1;
            frameworks
                .entry(parent_id)
                .or_default()
                .push(DocumentFramework {
                    id,
                    language_id: parent_id,
                    framework_name: framework.framework_name.clone(),
                    display_order: None,
                });
        }

        ResumeDocument {
            resume: DocumentResume {
                id: 0,
                name: parsed.name,
                profile_image_url: parsed.profile_image_url,
                location: parsed.location,
                email: parsed.email,
                github_url: parsed.github_url,
                mobile_number: parsed.mobile_number,
                executive_summary: parsed.executive_summary,
                is_public: parsed.is_public,
                video: parsed.video,
            },
            education,
            education_key_points,
            skills,
            work_experiences,
            work_experience_key_points,
            portfolio_projects,
            portfolio_key_points,
            portfolio_technologies,
            languages,
            frameworks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::{
        DatePrecision, ParsedEducation, ParsedEducationKeyPoint, ParsedFramework, ParsedLanguage,
        ParsedPortfolioKeyPoint, ParsedPortfolioProject, ParsedPortfolioTechnology, ParsedResume,
        ParsedSkill, ParsedWorkExperience, ParsedWorkExperienceKeyPoint,
    };
    use rocket::serde::json::serde_json;

    fn month_date(year: i32, month: u32) -> PartialDate {
        PartialDate {
            canonical: chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap(),
            precision: DatePrecision::Month,
        }
    }

    fn empty_parsed() -> ParsedResume {
        ParsedResume {
            name: "Jane".to_string(),
            profile_image_url: None,
            location: None,
            email: "jane@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: None,
            video: None,
            is_public: true,
            education: vec![],
            education_key_points: vec![],
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: vec![],
            portfolio_projects: vec![],
            portfolio_key_points: vec![],
            portfolio_technologies: vec![],
            languages: vec![],
            frameworks: vec![],
        }
    }

    #[test]
    fn mapping_mints_sequential_parent_ids() {
        let mut parsed = empty_parsed();
        parsed.education = vec![
            ParsedEducation {
                education_stage: "Bachelor's".to_string(),
                institution_name: "Uni A".to_string(),
                degree: None,
                start_date: month_date(2020, 9),
                end_date: Some(month_date(2024, 5)),
                description: None,
                display_order: Some(3),
            },
            ParsedEducation {
                education_stage: "Master's".to_string(),
                institution_name: "Uni B".to_string(),
                degree: Some("MSc".to_string()),
                start_date: month_date(2024, 9),
                end_date: None,
                description: None,
                display_order: None,
            },
        ];

        let document = ResumeDocument::from(parsed);
        assert_eq!(document.education.len(), 2);
        assert_eq!(document.education[0].id, 1);
        assert_eq!(document.education[1].id, 2);
        assert_eq!(document.education[0].resume_id, 0);
        assert!(document.education[0].active);
        assert_eq!(document.education[0].display_order, Some(3));
        assert_eq!(document.education[1].display_order, None);
    }

    #[test]
    fn child_map_keyed_by_minted_parent_id() {
        let mut parsed = empty_parsed();
        parsed.education = vec![
            ParsedEducation {
                education_stage: "A".to_string(),
                institution_name: "Uni A".to_string(),
                degree: None,
                start_date: month_date(2020, 9),
                end_date: None,
                description: None,
                display_order: None,
            },
            ParsedEducation {
                education_stage: "B".to_string(),
                institution_name: "Uni B".to_string(),
                degree: None,
                start_date: month_date(2024, 9),
                end_date: None,
                description: None,
                display_order: None,
            },
        ];
        parsed.education_key_points = vec![
            ParsedEducationKeyPoint {
                education_index: 0,
                key_point: "kp one".to_string(),
            },
            ParsedEducationKeyPoint {
                education_index: 1,
                key_point: "kp two".to_string(),
            },
            ParsedEducationKeyPoint {
                education_index: 0,
                key_point: "kp three".to_string(),
            },
        ];

        let document = ResumeDocument::from(parsed);
        let map = &document.education_key_points;
        assert_eq!(map.len(), 2);
        let first = &map[&1];
        let second = &map[&2];
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].key_point, "kp one");
        assert_eq!(first[1].key_point, "kp three");
        assert_eq!(second[0].key_point, "kp two");
        // Child ids are 1-based sequential across the whole map in document order.
        assert_eq!(first[0].id, 1);
        assert_eq!(second[0].id, 2);
        assert_eq!(first[1].id, 3);
        assert_eq!(first[0].education_id, 1);
        assert_eq!(second[0].education_id, 2);
        assert!(first[0].active);
        assert_eq!(first[0].display_order, None);
    }

    #[test]
    fn mapping_propagates_resume_fields_and_placeholders() {
        let mut parsed = empty_parsed();
        parsed.is_public = true;
        parsed.video = Some("https://video.example".to_string());
        parsed.executive_summary = Some("Summary text".to_string());
        parsed.skills = vec![
            ParsedSkill {
                skill_name: "Rust".to_string(),
                confidence_percentage: 90,
                display_order: Some(2),
            },
            ParsedSkill {
                skill_name: "Go".to_string(),
                confidence_percentage: 70,
                display_order: None,
            },
        ];

        let document = ResumeDocument::from(parsed);
        assert_eq!(document.resume.id, 0);
        assert!(document.resume.is_public);
        assert_eq!(
            document.resume.video.as_deref(),
            Some("https://video.example")
        );
        assert_eq!(
            document.resume.executive_summary.as_deref(),
            Some("Summary text")
        );
        assert_eq!(document.skills[0].id, 1);
        assert_eq!(document.skills[1].id, 2);
        assert_eq!(document.skills[0].resume_id, 0);
        assert_eq!(document.skills[0].display_order, Some(2));
    }

    #[test]
    fn dates_preserve_precision_in_document_json() {
        let mut parsed = empty_parsed();
        parsed.work_experiences = vec![ParsedWorkExperience {
            job_title: "Engineer".to_string(),
            company_name: Some("Corp".to_string()),
            start_date: month_date(2020, 1),
            end_date: None,
            description: None,
            display_order: None,
        }];

        let document = ResumeDocument::from(parsed);
        let json = serde_json::to_value(&document).unwrap();
        assert_eq!(json["work_experiences"][0]["start_date"], "2020-01");
        assert_eq!(
            json["work_experiences"][0]["end_date"],
            serde_json::Value::Null
        );
        assert_eq!(json["work_experiences"][0]["active"], true);
    }

    #[test]
    fn portfolio_and_language_child_maps_mint_ids() {
        let mut parsed = empty_parsed();
        parsed.portfolio_projects = vec![ParsedPortfolioProject {
            project_name: "Proj".to_string(),
            image_url: None,
            project_link: None,
            source_code_link: None,
            video_url: None,
            description: None,
            display_order: None,
        }];
        parsed.portfolio_key_points = vec![ParsedPortfolioKeyPoint {
            portfolio_project_index: 0,
            key_point: "did thing".to_string(),
        }];
        parsed.portfolio_technologies = vec![
            ParsedPortfolioTechnology {
                portfolio_project_index: 0,
                technology_name: "Rust".to_string(),
            },
            ParsedPortfolioTechnology {
                portfolio_project_index: 0,
                technology_name: "Rocket".to_string(),
            },
        ];
        parsed.languages = vec![
            ParsedLanguage {
                language_name: "Rust".to_string(),
                display_order: None,
            },
            ParsedLanguage {
                language_name: "Go".to_string(),
                display_order: None,
            },
        ];
        parsed.frameworks = vec![ParsedFramework {
            language_index: 1,
            framework_name: "Gin".to_string(),
        }];
        parsed.work_experiences = vec![ParsedWorkExperience {
            job_title: "Dev".to_string(),
            company_name: None,
            start_date: month_date(2021, 6),
            end_date: Some(month_date(2022, 6)),
            description: None,
            display_order: None,
        }];
        parsed.work_experience_key_points = vec![ParsedWorkExperienceKeyPoint {
            work_experience_index: 0,
            key_point: "shipped".to_string(),
        }];

        let document = ResumeDocument::from(parsed);

        let techs = &document.portfolio_technologies[&1];
        assert_eq!(techs.len(), 2);
        assert_eq!(techs[0].id, 1);
        assert_eq!(techs[1].id, 2);
        assert_eq!(techs[0].portfolio_project_id, 1);
        assert_eq!(document.portfolio_key_points[&1][0].portfolio_project_id, 1);
        assert_eq!(document.portfolio_key_points[&1][0].id, 1);

        // Framework map keyed by minted language id (second language → id 2).
        let fws = &document.frameworks[&2];
        assert_eq!(fws[0].language_id, 2);
        assert_eq!(fws[0].id, 1);

        let work_kps = &document.work_experience_key_points[&1];
        assert_eq!(work_kps[0].work_experience_id, 1);
        assert_eq!(work_kps[0].id, 1);
    }

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
