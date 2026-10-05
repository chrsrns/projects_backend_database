use std::collections::HashMap;

use domain::models::{
    EducationKeyPoint, Framework, FullResume, PortfolioKeyPoint, PortfolioTechnology,
    WorkExperienceKeyPoint,
};
use shared::markdown;

use crate::{
    error::ApplicationError,
    resume::{
        common::find_accessible_resume, education, frameworks, languages, portfolio_projects,
        skills, work_experiences,
    },
};

pub fn export_resume_markdown(
    resume_id: i32,
    user_id_value: Option<i32>,
) -> Result<String, ApplicationError> {
    let resume = find_accessible_resume(resume_id, user_id_value)?;

    let education_items = education::read::list_educations(resume_id, user_id_value)?;
    let mut education_key_points: HashMap<i32, Vec<EducationKeyPoint>> = HashMap::new();
    for edu in &education_items {
        let kps = education::read::list_education_key_points(resume_id, edu.id, user_id_value)?;
        education_key_points.insert(edu.id, kps);
    }

    let skills = skills::read::list_skills(resume_id, user_id_value)?;

    let work_experience_items =
        work_experiences::read::list_work_experiences(resume_id, user_id_value)?;
    let mut work_experience_key_points: HashMap<i32, Vec<WorkExperienceKeyPoint>> = HashMap::new();
    for work in &work_experience_items {
        let kps = work_experiences::read::list_work_experience_key_points(
            resume_id,
            work.id,
            user_id_value,
        )?;
        work_experience_key_points.insert(work.id, kps);
    }

    let portfolio_projects_items =
        portfolio_projects::read::list_portfolio_projects(resume_id, user_id_value)?;
    let mut portfolio_key_points: HashMap<i32, Vec<PortfolioKeyPoint>> = HashMap::new();
    let mut portfolio_technologies: HashMap<i32, Vec<PortfolioTechnology>> = HashMap::new();
    for project in &portfolio_projects_items {
        let kps = portfolio_projects::read::list_portfolio_key_points(
            resume_id,
            project.id,
            user_id_value,
        )?;
        portfolio_key_points.insert(project.id, kps);
        let techs = portfolio_projects::read::list_portfolio_technologies(
            resume_id,
            project.id,
            user_id_value,
        )?;
        portfolio_technologies.insert(project.id, techs);
    }

    let language_items = languages::read::list_languages(resume_id, user_id_value)?;
    let mut frameworks: HashMap<i32, Vec<Framework>> = HashMap::new();
    for language in &language_items {
        let fws = frameworks::read::list_frameworks(resume_id, language.id, user_id_value)?;
        frameworks.insert(language.id, fws);
    }

    // `show_variant_tag` is owner-only output; the front-matter marker is
    // always emitted so a re-import can target this exact row.
    let viewer_is_owner = resume.created_by.is_some() && resume.created_by == user_id_value;

    let full_resume = FullResume {
        resume,
        education: education_items,
        education_key_points,
        skills,
        work_experiences: work_experience_items,
        work_experience_key_points,
        portfolio_projects: portfolio_projects_items,
        portfolio_key_points,
        portfolio_technologies,
        languages: language_items,
        frameworks,
    };

    Ok(markdown::resume_to_markdown(&full_resume, viewer_is_owner))
}
