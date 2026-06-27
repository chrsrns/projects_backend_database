use diesel::prelude::*;
use domain::models::{
    NewEducation, NewEducationKeyPoint, NewFramework, NewLanguage, NewPortfolioKeyPoint,
    NewPortfolioProject, NewPortfolioTechnology, NewResume, NewSkill, NewWorkExperience,
    NewWorkExperienceKeyPoint, Resume,
};
use infrastructure::run_in_transaction;
use shared::markdown;

use crate::{error::ApplicationError, resume::common::app_err_from_diesel_err};

/// Resolves an index into a parent ID map, returning `BadRequest` if the
/// index is out of bounds (V21).
///
/// `markdown_to_resume` already calls `validate_child_indices` and returns an
/// error for any out-of-bounds reference in the parsed markdown, so this
/// helper functions as a second, defensive check at the import layer.  Any
/// code path that would previously have silently skipped a child row now
/// returns a `BadRequest` instead.
pub fn resolve_parent_index(
    map: &[i32],
    index: usize,
    context: &str,
) -> Result<i32, ApplicationError> {
    map.get(index).copied().ok_or_else(|| {
        ApplicationError::BadRequest(format!(
            "{} references out-of-bounds parent index {} (only {} parents available)",
            context,
            index,
            map.len()
        ))
    })
}

pub fn import_resume_markdown(
    markdown: &str,
    user_id_value: i32,
) -> Result<Resume, ApplicationError> {
    let full_resume = markdown::markdown_to_resume(markdown).map_err(|err| match err {
        markdown::MarkdownError::InvalidMarkdown(msg) => ApplicationError::BadRequest(msg),
    })?;

    // Pre-flight bounds check (V21): validate all child → parent index
    // references before opening the database transaction.  This lets us
    // return BadRequest immediately without needing to surface the error
    // through the diesel transaction's error type.
    let n_edu = full_resume.education.len();
    for kp in &full_resume.education_key_points {
        if kp.education_index >= n_edu {
            return Err(ApplicationError::BadRequest(format!(
                "Education key point references out-of-bounds education index {} (only {} available)",
                kp.education_index, n_edu
            )));
        }
    }
    let n_work = full_resume.work_experiences.len();
    for kp in &full_resume.work_experience_key_points {
        if kp.work_experience_index >= n_work {
            return Err(ApplicationError::BadRequest(format!(
                "Work experience key point references out-of-bounds work experience index {} (only {} available)",
                kp.work_experience_index, n_work
            )));
        }
    }
    let n_projects = full_resume.portfolio_projects.len();
    for kp in &full_resume.portfolio_key_points {
        if kp.portfolio_project_index >= n_projects {
            return Err(ApplicationError::BadRequest(format!(
                "Portfolio key point references out-of-bounds project index {} (only {} available)",
                kp.portfolio_project_index, n_projects
            )));
        }
    }
    for tech in &full_resume.portfolio_technologies {
        if tech.portfolio_project_index >= n_projects {
            return Err(ApplicationError::BadRequest(format!(
                "Portfolio technology references out-of-bounds project index {} (only {} available)",
                tech.portfolio_project_index, n_projects
            )));
        }
    }
    let n_languages = full_resume.languages.len();
    for fw in &full_resume.frameworks {
        if fw.language_index >= n_languages {
            return Err(ApplicationError::BadRequest(format!(
                "Framework references out-of-bounds language index {} (only {} available)",
                fw.language_index, n_languages
            )));
        }
    }

    let mut conn = infrastructure::establish_connection();
    run_in_transaction(&mut conn, |conn| {
        use domain::schema::{
            education, education_key_points, frameworks, languages, portfolio_key_points,
            portfolio_projects, portfolio_technologies, resumes, skills,
            work_experience_key_points, work_experiences,
        };

        let new_resume = NewResume {
            name: full_resume.name,
            profile_image_url: full_resume.profile_image_url,
            location: full_resume.location,
            email: full_resume.email,
            github_url: full_resume.github_url,
            mobile_number: full_resume.mobile_number,
            created_by: Some(user_id_value),
            is_public: full_resume.is_public,
        };

        let resume: Resume = diesel::insert_into(resumes::table)
            .values(&new_resume)
            .get_result::<Resume>(conn)?;

        let mut education_id_map: Vec<i32> = Vec::new();
        for (idx, edu) in full_resume.education.iter().enumerate() {
            let new_edu = NewEducation {
                resume_id: resume.id,
                education_stage: edu.education_stage.clone(),
                institution_name: edu.institution_name.clone(),
                degree: edu.degree.clone(),
                start_date: edu.start_date,
                end_date: edu.end_date,
                description: edu.description.clone(),
                display_order: edu.display_order.or(Some(idx as i32)),
            };
            let inserted: domain::models::Education = diesel::insert_into(education::table)
                .values(&new_edu)
                .get_result::<domain::models::Education>(conn)?;
            education_id_map.push(inserted.id);
        }

        // Indices were validated above; unwrap is safe here.
        for kp in &full_resume.education_key_points {
            let edu_id = education_id_map[kp.education_index];
            let new_kp = NewEducationKeyPoint {
                education_id: edu_id,
                key_point: kp.key_point.clone(),
                display_order: None,
            };
            diesel::insert_into(education_key_points::table)
                .values(&new_kp)
                .execute(conn)?;
        }

        for (idx, skill) in full_resume.skills.iter().enumerate() {
            let new_skill = NewSkill {
                resume_id: resume.id,
                skill_name: skill.skill_name.clone(),
                confidence_percentage: skill.confidence_percentage,
                display_order: skill.display_order.or(Some(idx as i32)),
            };
            diesel::insert_into(skills::table)
                .values(&new_skill)
                .execute(conn)?;
        }

        let mut work_id_map: Vec<i32> = Vec::new();
        for (idx, work) in full_resume.work_experiences.iter().enumerate() {
            let new_work = NewWorkExperience {
                resume_id: resume.id,
                job_title: work.job_title.clone(),
                company_name: work.company_name.clone(),
                start_date: work.start_date,
                end_date: work.end_date,
                description: work.description.clone(),
                display_order: work.display_order.or(Some(idx as i32)),
            };
            let inserted: domain::models::WorkExperience =
                diesel::insert_into(work_experiences::table)
                    .values(&new_work)
                    .get_result::<domain::models::WorkExperience>(conn)?;
            work_id_map.push(inserted.id);
        }

        // Indices were validated above; unwrap is safe here.
        for kp in &full_resume.work_experience_key_points {
            let work_id = work_id_map[kp.work_experience_index];
            let new_kp = NewWorkExperienceKeyPoint {
                work_experience_id: work_id,
                key_point: kp.key_point.clone(),
                display_order: None,
            };
            diesel::insert_into(work_experience_key_points::table)
                .values(&new_kp)
                .execute(conn)?;
        }

        let mut project_id_map: Vec<i32> = Vec::new();
        for (idx, project) in full_resume.portfolio_projects.iter().enumerate() {
            let new_project = NewPortfolioProject {
                resume_id: resume.id,
                project_name: project.project_name.clone(),
                image_url: project.image_url.clone(),
                project_link: project.project_link.clone(),
                source_code_link: project.source_code_link.clone(),
                description: project.description.clone(),
                display_order: project.display_order.or(Some(idx as i32)),
            };
            let inserted: domain::models::PortfolioProject =
                diesel::insert_into(portfolio_projects::table)
                    .values(&new_project)
                    .get_result::<domain::models::PortfolioProject>(conn)?;
            project_id_map.push(inserted.id);
        }

        // Indices were validated above; unwrap is safe here.
        for kp in &full_resume.portfolio_key_points {
            let project_id = project_id_map[kp.portfolio_project_index];
            let new_kp = NewPortfolioKeyPoint {
                portfolio_project_id: project_id,
                key_point: kp.key_point.clone(),
                display_order: None,
            };
            diesel::insert_into(portfolio_key_points::table)
                .values(&new_kp)
                .execute(conn)?;
        }

        for tech in &full_resume.portfolio_technologies {
            let project_id = project_id_map[tech.portfolio_project_index];
            let new_tech = NewPortfolioTechnology {
                portfolio_project_id: project_id,
                technology_name: tech.technology_name.clone(),
                display_order: None,
            };
            diesel::insert_into(portfolio_technologies::table)
                .values(&new_tech)
                .execute(conn)?;
        }

        let mut language_id_map: Vec<i32> = Vec::new();
        for (idx, language) in full_resume.languages.iter().enumerate() {
            let new_language = NewLanguage {
                resume_id: resume.id,
                language_name: language.language_name.clone(),
                display_order: language.display_order.or(Some(idx as i32)),
            };
            let inserted: domain::models::Language = diesel::insert_into(languages::table)
                .values(&new_language)
                .get_result::<domain::models::Language>(conn)?;
            language_id_map.push(inserted.id);
        }

        // Indices were validated above; unwrap is safe here.
        for fw in &full_resume.frameworks {
            let language_id = language_id_map[fw.language_index];
            let new_fw = NewFramework {
                language_id,
                framework_name: fw.framework_name.clone(),
                display_order: None,
            };
            diesel::insert_into(frameworks::table)
                .values(&new_fw)
                .execute(conn)?;
        }

        Ok(resume)
    })
    .map_err(app_err_from_diesel_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_parent_index_in_bounds() {
        let map = vec![10, 20, 30];
        assert_eq!(resolve_parent_index(&map, 0, "Test").unwrap(), 10);
        assert_eq!(resolve_parent_index(&map, 2, "Test").unwrap(), 30);
    }

    #[test]
    fn test_resolve_parent_index_out_of_bounds_returns_bad_request() {
        let map = vec![10, 20, 30];
        let result = resolve_parent_index(&map, 5, "Education key point");
        match result {
            Err(ApplicationError::BadRequest(msg)) => {
                assert!(msg.contains("Education key point"));
                assert!(msg.contains("index 5"));
                assert!(msg.contains("3 parents available"));
            }
            other => panic!("Expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_parent_index_empty_map_returns_bad_request() {
        let map: Vec<i32> = vec![];
        let result = resolve_parent_index(&map, 0, "Framework");
        match result {
            Err(ApplicationError::BadRequest(msg)) => {
                assert!(msg.contains("Framework"));
                assert!(msg.contains("0 parents available"));
            }
            other => panic!("Expected BadRequest, got {:?}", other),
        }
    }
}
