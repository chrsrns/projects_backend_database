use std::collections::HashMap;
use std::str::FromStr;

use chrono::NaiveDate;
use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, Framework, FullResume, Language, PartialDate, PortfolioKeyPoint,
    PortfolioProject, PortfolioTechnology, Resume, ResumeSnapshot, SNAPSHOT_SCHEMA_VERSION, Skill,
    SnapshotEducation, SnapshotEducationKeyPoint, SnapshotFramework, SnapshotLanguage,
    SnapshotPortfolioKeyPoint, SnapshotPortfolioProject, SnapshotPortfolioTechnology,
    SnapshotSkill, SnapshotWorkExperience, SnapshotWorkExperienceKeyPoint, Visibility,
    WorkExperience, WorkExperienceKeyPoint,
};
use infrastructure::DbConnection;
use uuid::Uuid;

use domain::models::DatePrecision;

/// Reads the whole resume aggregate on an already checked-out connection.
///
/// The caller owns the connection so this can run inside a mutation's
/// transaction, which is where the pre-state has to be read (V197, V239).
///
/// Collections are ordered by child id, which is stable across edits. That
/// keeps a positional diff meaningful: reordering a section changes the
/// `display_order` field rather than every position in the collection.
pub fn load_full_resume(
    conn: &mut DbConnection,
    resume_id: Uuid,
) -> QueryResult<Option<FullResume>> {
    use domain::schema::{
        education, education_key_points, frameworks, languages, portfolio_key_points,
        portfolio_projects, portfolio_technologies, resumes, skills, work_experience_key_points,
        work_experiences,
    };

    let Some(resume) = resumes::table
        .find(resume_id)
        .first::<Resume>(conn)
        .optional()?
    else {
        return Ok(None);
    };

    let mut education_items = education::table
        .filter(education::resume_id.eq(resume_id))
        .load::<Education>(conn)?;
    education_items.sort_by_key(|item| item.id);

    let education_ids: Vec<i32> = education_items.iter().map(|item| item.id).collect();
    let mut education_key_point_items: Vec<EducationKeyPoint> = if education_ids.is_empty() {
        Vec::new()
    } else {
        education_key_points::table
            .filter(education_key_points::education_id.eq_any(&education_ids))
            .load(conn)?
    };
    education_key_point_items.sort_by_key(|item| item.id);

    let mut skills_items = skills::table
        .filter(skills::resume_id.eq(resume_id))
        .load::<Skill>(conn)?;
    skills_items.sort_by_key(|item| item.id);

    let mut work_items = work_experiences::table
        .filter(work_experiences::resume_id.eq(resume_id))
        .load::<WorkExperience>(conn)?;
    work_items.sort_by_key(|item| item.id);

    let work_ids: Vec<i32> = work_items.iter().map(|item| item.id).collect();
    let mut work_key_point_items: Vec<WorkExperienceKeyPoint> = if work_ids.is_empty() {
        Vec::new()
    } else {
        work_experience_key_points::table
            .filter(work_experience_key_points::work_experience_id.eq_any(&work_ids))
            .load(conn)?
    };
    work_key_point_items.sort_by_key(|item| item.id);

    let mut project_items = portfolio_projects::table
        .filter(portfolio_projects::resume_id.eq(resume_id))
        .load::<PortfolioProject>(conn)?;
    project_items.sort_by_key(|item| item.id);

    let project_ids: Vec<i32> = project_items.iter().map(|item| item.id).collect();
    let mut project_key_point_items: Vec<PortfolioKeyPoint> = if project_ids.is_empty() {
        Vec::new()
    } else {
        portfolio_key_points::table
            .filter(portfolio_key_points::portfolio_project_id.eq_any(&project_ids))
            .load(conn)?
    };
    project_key_point_items.sort_by_key(|item| item.id);

    let mut technology_items: Vec<PortfolioTechnology> = if project_ids.is_empty() {
        Vec::new()
    } else {
        portfolio_technologies::table
            .filter(portfolio_technologies::portfolio_project_id.eq_any(&project_ids))
            .load(conn)?
    };
    technology_items.sort_by_key(|item| item.id);

    let mut language_items = languages::table
        .filter(languages::resume_id.eq(resume_id))
        .load::<Language>(conn)?;
    language_items.sort_by_key(|item| item.id);

    let language_ids: Vec<i32> = language_items.iter().map(|item| item.id).collect();
    let mut framework_items: Vec<Framework> = if language_ids.is_empty() {
        Vec::new()
    } else {
        frameworks::table
            .filter(frameworks::language_id.eq_any(&language_ids))
            .load(conn)?
    };
    framework_items.sort_by_key(|item| item.id);

    Ok(Some(FullResume {
        resume,
        education: education_items,
        education_key_points: group_by(education_key_point_items, |item| item.education_id),
        skills: skills_items,
        work_experiences: work_items,
        work_experience_key_points: group_by(work_key_point_items, |item| item.work_experience_id),
        portfolio_projects: project_items,
        portfolio_key_points: group_by(project_key_point_items, |item| item.portfolio_project_id),
        portfolio_technologies: group_by(technology_items, |item| item.portfolio_project_id),
        languages: language_items,
        frameworks: group_by(framework_items, |item| item.language_id),
    }))
}

fn group_by<T, K: std::hash::Hash + Eq>(
    items: Vec<T>,
    key: impl Fn(&T) -> K,
) -> HashMap<K, Vec<T>> {
    let mut grouped: HashMap<K, Vec<T>> = HashMap::new();
    for item in items {
        grouped.entry(key(&item)).or_default().push(item);
    }
    grouped
}

/// Rebuilds the stored partial date from a canonical column plus its precision
/// column. An unreadable precision falls back to `day`, matching how an
/// existing row without precision is presented (V55).
fn partial_date(canonical: NaiveDate, precision: Option<&str>) -> PartialDate {
    PartialDate {
        canonical,
        precision: precision
            .and_then(|value| DatePrecision::from_str(value).ok())
            .unwrap_or(DatePrecision::Day),
    }
}

fn optional_partial_date(
    canonical: Option<NaiveDate>,
    precision: Option<&str>,
) -> Option<PartialDate> {
    canonical.map(|date| partial_date(date, precision))
}

/// Maps a loaded aggregate onto the restorable document shape.
pub fn snapshot_from_full_resume(full: &FullResume) -> ResumeSnapshot {
    ResumeSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        name: full.resume.name.clone(),
        profile_image_url: full.resume.profile_image_url.clone(),
        location: full.resume.location.clone(),
        email: full.resume.email.clone(),
        github_url: full.resume.github_url.clone(),
        mobile_number: full.resume.mobile_number.clone(),
        executive_summary: full.resume.executive_summary.clone(),
        video: full.resume.video.clone(),
        visibility: Visibility::from_str(&full.resume.visibility).unwrap_or(Visibility::Private),
        company_name: full.resume.company_name.clone(),
        role_title: full.resume.role_title.clone(),
        target_date: optional_partial_date(
            full.resume.target_date,
            full.resume.target_date_precision.as_deref(),
        ),
        target_date_precision: full.resume.target_date_precision.clone(),
        job_description: full.resume.job_description.clone(),
        variant_label: full.resume.variant_label.clone(),
        show_variant_tag: full.resume.show_variant_tag,
        education: full
            .education
            .iter()
            .map(|item| SnapshotEducation {
                id: item.id,
                education_stage: item.education_stage.clone(),
                institution_name: item.institution_name.clone(),
                degree: item.degree.clone(),
                start_date: partial_date(item.start_date, Some(&item.start_date_precision)),
                end_date: optional_partial_date(item.end_date, item.end_date_precision.as_deref()),
                description: item.description.clone(),
                display_order: item.display_order,
                active: item.active,
            })
            .collect(),
        education_key_points: full
            .education_key_points
            .iter()
            .map(|(education_id, items)| {
                (
                    *education_id,
                    items
                        .iter()
                        .map(|item| SnapshotEducationKeyPoint {
                            id: item.id,
                            key_point: item.key_point.clone(),
                            display_order: item.display_order,
                            active: item.active,
                        })
                        .collect(),
                )
            })
            .collect(),
        skills: full
            .skills
            .iter()
            .map(|item| SnapshotSkill {
                id: item.id,
                skill_name: item.skill_name.clone(),
                confidence_percentage: item.confidence_percentage,
                display_order: item.display_order,
            })
            .collect(),
        work_experiences: full
            .work_experiences
            .iter()
            .map(|item| SnapshotWorkExperience {
                id: item.id,
                job_title: item.job_title.clone(),
                company_name: item.company_name.clone(),
                start_date: partial_date(item.start_date, Some(&item.start_date_precision)),
                end_date: optional_partial_date(item.end_date, item.end_date_precision.as_deref()),
                description: item.description.clone(),
                display_order: item.display_order,
                active: item.active,
            })
            .collect(),
        work_experience_key_points: full
            .work_experience_key_points
            .iter()
            .map(|(work_id, items)| {
                (
                    *work_id,
                    items
                        .iter()
                        .map(|item| SnapshotWorkExperienceKeyPoint {
                            id: item.id,
                            key_point: item.key_point.clone(),
                            display_order: item.display_order,
                            active: item.active,
                        })
                        .collect(),
                )
            })
            .collect(),
        portfolio_projects: full
            .portfolio_projects
            .iter()
            .map(|item| SnapshotPortfolioProject {
                id: item.id,
                project_name: item.project_name.clone(),
                image_url: item.image_url.clone(),
                project_link: item.project_link.clone(),
                source_code_link: item.source_code_link.clone(),
                video_url: item.video_url.clone(),
                description: item.description.clone(),
                display_order: item.display_order,
                active: item.active,
            })
            .collect(),
        portfolio_key_points: full
            .portfolio_key_points
            .iter()
            .map(|(project_id, items)| {
                (
                    *project_id,
                    items
                        .iter()
                        .map(|item| SnapshotPortfolioKeyPoint {
                            id: item.id,
                            key_point: item.key_point.clone(),
                            display_order: item.display_order,
                            active: item.active,
                        })
                        .collect(),
                )
            })
            .collect(),
        portfolio_technologies: full
            .portfolio_technologies
            .iter()
            .map(|(project_id, items)| {
                (
                    *project_id,
                    items
                        .iter()
                        .map(|item| SnapshotPortfolioTechnology {
                            id: item.id,
                            technology_name: item.technology_name.clone(),
                            display_order: item.display_order,
                            active: item.active,
                        })
                        .collect(),
                )
            })
            .collect(),
        languages: full
            .languages
            .iter()
            .map(|item| SnapshotLanguage {
                id: item.id,
                language_name: item.language_name.clone(),
                display_order: item.display_order,
            })
            .collect(),
        frameworks: full
            .frameworks
            .iter()
            .map(|(language_id, items)| {
                (
                    *language_id,
                    items
                        .iter()
                        .map(|item| SnapshotFramework {
                            id: item.id,
                            framework_name: item.framework_name.clone(),
                            display_order: item.display_order,
                        })
                        .collect(),
                )
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn resume_row() -> Resume {
        Resume {
            id: Uuid::nil(),
            name: "Snapshot Subject".to_string(),
            profile_image_url: None,
            location: Some("Here".to_string()),
            email: "snapshot@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            created_by: 1,
            visibility: "unlisted".to_string(),
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

    fn education_row(id: i32, display_order: Option<i32>) -> Education {
        Education {
            id,
            education_stage: "Bachelor".to_string(),
            institution_name: "University".to_string(),
            degree: None,
            start_date: NaiveDate::from_ymd_opt(2020, 1, 1).expect("date"),
            end_date: None,
            description: None,
            display_order,
            created_at: Utc::now().naive_utc(),
            active: true,
            start_date_precision: "day".to_string(),
            end_date_precision: None,
            resume_id: Uuid::nil(),
        }
    }

    fn empty_full_resume() -> FullResume {
        FullResume {
            resume: resume_row(),
            education: Vec::new(),
            education_key_points: HashMap::new(),
            skills: Vec::new(),
            work_experiences: Vec::new(),
            work_experience_key_points: HashMap::new(),
            portfolio_projects: Vec::new(),
            portfolio_key_points: HashMap::new(),
            portfolio_technologies: HashMap::new(),
            languages: Vec::new(),
            frameworks: HashMap::new(),
        }
    }

    #[test]
    fn snapshot_maps_content_columns_and_excludes_identity() {
        let snapshot = snapshot_from_full_resume(&empty_full_resume());
        let value = snapshot.to_json().expect("serialize");

        assert_eq!(snapshot.name, "Snapshot Subject");
        assert_eq!(snapshot.visibility, Visibility::Unlisted);
        assert_eq!(snapshot.schema_version, SNAPSHOT_SCHEMA_VERSION);

        for field in [
            "id",
            "created_at",
            "updated_at",
            "created_by",
            "base_resume_id",
        ] {
            assert!(
                value.get(field).is_none(),
                "the snapshot must not carry {}",
                field
            );
        }
    }

    #[test]
    fn snapshot_maps_dates_with_their_precision() {
        let mut full = empty_full_resume();
        full.resume.target_date = NaiveDate::from_ymd_opt(2026, 3, 4);
        full.resume.target_date_precision = Some("month".to_string());
        full.education = vec![education_row(1, Some(1))];

        let snapshot = snapshot_from_full_resume(&full);

        assert_eq!(
            snapshot.target_date.expect("target date").precision,
            DatePrecision::Month
        );
        assert_eq!(
            snapshot.education[0].start_date.precision,
            DatePrecision::Day
        );
        assert_eq!(
            snapshot.education[0].end_date, None,
            "a missing end date stays absent"
        );
    }

    #[test]
    fn snapshot_keeps_child_order_stable_by_id() {
        let mut full = empty_full_resume();
        full.education = vec![education_row(5, Some(1)), education_row(2, Some(9))];

        let snapshot = snapshot_from_full_resume(&full);

        assert_eq!(
            snapshot
                .education
                .iter()
                .map(|item| item.id)
                .collect::<Vec<i32>>(),
            vec![5, 2],
            "the caller orders by id; the mapping preserves that order"
        );
    }
}
