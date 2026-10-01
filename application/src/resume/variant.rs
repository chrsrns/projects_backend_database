use std::collections::HashMap;

use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, Framework, Language, NewEducation, NewEducationKeyPoint,
    NewFramework, NewLanguage, NewPortfolioKeyPoint, NewPortfolioProject, NewPortfolioTechnology,
    NewResume, NewSkill, NewVariantRequest, NewWorkExperience, NewWorkExperienceKeyPoint,
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume, Skill, WorkExperience,
    WorkExperienceKeyPoint,
};
use infrastructure::run_in_transaction;

use crate::error::ApplicationError;
use crate::resume::common::{
    app_err_from_diesel_err, find_resume, partial_date_to_columns, validate_variant_metadata,
};

pub fn create_variant(
    user_id_value: i32,
    base_id: i32,
    request: NewVariantRequest,
) -> Result<Resume, ApplicationError> {
    let base = find_resume(base_id)?;

    match base.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => return Err(ApplicationError::Forbidden),
    }

    if base.base_resume_id.is_some() {
        return Err(ApplicationError::BadRequest(
            "A variant cannot be created from another variant".to_string(),
        ));
    }

    let metadata = validate_variant_metadata(
        request.company_name,
        request.role_title,
        request.variant_label,
        request.job_description,
    )?;

    let (target_date, target_date_precision) = match request.target_date {
        Some(date) => {
            let (canonical, precision) = partial_date_to_columns(date);
            (Some(canonical), Some(precision))
        }
        None => (None, None),
    };

    let new_resume = NewResume {
        name: base.name.clone(),
        profile_image_url: base.profile_image_url.clone(),
        location: base.location.clone(),
        email: base.email.clone(),
        github_url: base.github_url.clone(),
        mobile_number: base.mobile_number.clone(),
        executive_summary: base.executive_summary.clone(),
        video: base.video.clone(),
        created_by: Some(user_id_value),
        is_public: request.is_public.unwrap_or(base.is_public),
        base_resume_id: Some(base_id),
        company_name: metadata.company_name,
        role_title: metadata.role_title,
        target_date,
        target_date_precision,
        job_description: metadata.job_description,
        variant_label: metadata.variant_label,
        show_variant_tag: request.show_variant_tag.unwrap_or(true),
    };

    let mut conn = infrastructure::establish_connection();

    run_in_transaction(&mut conn, move |conn| {
        use domain::schema::{
            education, education_key_points, frameworks, languages, portfolio_key_points,
            portfolio_projects, portfolio_technologies, resumes, skills,
            work_experience_key_points, work_experiences,
        };

        let resume = diesel::insert_into(resumes::table)
            .values(&new_resume)
            .get_result::<Resume>(conn)?;

        let educations = education::table
            .filter(education::resume_id.eq(base_id))
            .load::<Education>(conn)?;
        let mut education_id_map: HashMap<i32, i32> = HashMap::new();
        for edu in &educations {
            let new_edu = NewEducation {
                resume_id: resume.id,
                education_stage: edu.education_stage.clone(),
                institution_name: edu.institution_name.clone(),
                degree: edu.degree.clone(),
                start_date: edu.start_date,
                start_date_precision: edu.start_date_precision.clone(),
                end_date: edu.end_date,
                end_date_precision: edu.end_date_precision.clone(),
                description: edu.description.clone(),
                display_order: edu.display_order,
                active: edu.active,
            };
            let inserted = diesel::insert_into(education::table)
                .values(&new_edu)
                .get_result::<Education>(conn)?;
            education_id_map.insert(edu.id, inserted.id);
        }

        let education_ids: Vec<i32> = educations.iter().map(|edu| edu.id).collect();
        if !education_ids.is_empty() {
            let key_points = education_key_points::table
                .filter(education_key_points::education_id.eq_any(&education_ids))
                .load::<EducationKeyPoint>(conn)?;
            for kp in &key_points {
                let new_kp = NewEducationKeyPoint {
                    education_id: education_id_map[&kp.education_id],
                    key_point: kp.key_point.clone(),
                    display_order: kp.display_order,
                    active: kp.active,
                };
                diesel::insert_into(education_key_points::table)
                    .values(&new_kp)
                    .execute(conn)?;
            }
        }

        let base_skills = skills::table
            .filter(skills::resume_id.eq(base_id))
            .load::<Skill>(conn)?;
        for skill in &base_skills {
            let new_skill = NewSkill {
                resume_id: resume.id,
                skill_name: skill.skill_name.clone(),
                confidence_percentage: skill.confidence_percentage,
                display_order: skill.display_order,
            };
            diesel::insert_into(skills::table)
                .values(&new_skill)
                .execute(conn)?;
        }

        let work_items = work_experiences::table
            .filter(work_experiences::resume_id.eq(base_id))
            .load::<WorkExperience>(conn)?;
        let mut work_id_map: HashMap<i32, i32> = HashMap::new();
        for work in &work_items {
            let new_work = NewWorkExperience {
                resume_id: resume.id,
                job_title: work.job_title.clone(),
                company_name: work.company_name.clone(),
                start_date: work.start_date,
                start_date_precision: work.start_date_precision.clone(),
                end_date: work.end_date,
                end_date_precision: work.end_date_precision.clone(),
                description: work.description.clone(),
                display_order: work.display_order,
                active: work.active,
            };
            let inserted = diesel::insert_into(work_experiences::table)
                .values(&new_work)
                .get_result::<WorkExperience>(conn)?;
            work_id_map.insert(work.id, inserted.id);
        }

        let work_ids: Vec<i32> = work_items.iter().map(|work| work.id).collect();
        if !work_ids.is_empty() {
            let key_points = work_experience_key_points::table
                .filter(work_experience_key_points::work_experience_id.eq_any(&work_ids))
                .load::<WorkExperienceKeyPoint>(conn)?;
            for kp in &key_points {
                let new_kp = NewWorkExperienceKeyPoint {
                    work_experience_id: work_id_map[&kp.work_experience_id],
                    key_point: kp.key_point.clone(),
                    display_order: kp.display_order,
                    active: kp.active,
                };
                diesel::insert_into(work_experience_key_points::table)
                    .values(&new_kp)
                    .execute(conn)?;
            }
        }

        let projects = portfolio_projects::table
            .filter(portfolio_projects::resume_id.eq(base_id))
            .load::<PortfolioProject>(conn)?;
        let mut project_id_map: HashMap<i32, i32> = HashMap::new();
        for project in &projects {
            let new_project = NewPortfolioProject {
                resume_id: resume.id,
                project_name: project.project_name.clone(),
                image_url: project.image_url.clone(),
                project_link: project.project_link.clone(),
                source_code_link: project.source_code_link.clone(),
                video_url: project.video_url.clone(),
                description: project.description.clone(),
                display_order: project.display_order,
                active: project.active,
            };
            let inserted = diesel::insert_into(portfolio_projects::table)
                .values(&new_project)
                .get_result::<PortfolioProject>(conn)?;
            project_id_map.insert(project.id, inserted.id);
        }

        let project_ids: Vec<i32> = projects.iter().map(|project| project.id).collect();
        if !project_ids.is_empty() {
            let key_points = portfolio_key_points::table
                .filter(portfolio_key_points::portfolio_project_id.eq_any(&project_ids))
                .load::<PortfolioKeyPoint>(conn)?;
            for kp in &key_points {
                let new_kp = NewPortfolioKeyPoint {
                    portfolio_project_id: project_id_map[&kp.portfolio_project_id],
                    key_point: kp.key_point.clone(),
                    display_order: kp.display_order,
                    active: kp.active,
                };
                diesel::insert_into(portfolio_key_points::table)
                    .values(&new_kp)
                    .execute(conn)?;
            }

            let technologies = portfolio_technologies::table
                .filter(portfolio_technologies::portfolio_project_id.eq_any(&project_ids))
                .load::<PortfolioTechnology>(conn)?;
            for tech in &technologies {
                let new_tech = NewPortfolioTechnology {
                    portfolio_project_id: project_id_map[&tech.portfolio_project_id],
                    technology_name: tech.technology_name.clone(),
                    display_order: tech.display_order,
                    active: tech.active,
                };
                diesel::insert_into(portfolio_technologies::table)
                    .values(&new_tech)
                    .execute(conn)?;
            }
        }

        let base_languages = languages::table
            .filter(languages::resume_id.eq(base_id))
            .load::<Language>(conn)?;
        let mut language_id_map: HashMap<i32, i32> = HashMap::new();
        for language in &base_languages {
            let new_language = NewLanguage {
                resume_id: resume.id,
                language_name: language.language_name.clone(),
                display_order: language.display_order,
            };
            let inserted = diesel::insert_into(languages::table)
                .values(&new_language)
                .get_result::<Language>(conn)?;
            language_id_map.insert(language.id, inserted.id);
        }

        let language_ids: Vec<i32> = base_languages.iter().map(|language| language.id).collect();
        if !language_ids.is_empty() {
            let frameworks_list = frameworks::table
                .filter(frameworks::language_id.eq_any(&language_ids))
                .load::<Framework>(conn)?;
            for framework in &frameworks_list {
                let new_framework = NewFramework {
                    language_id: language_id_map[&framework.language_id],
                    framework_name: framework.framework_name.clone(),
                    display_order: framework.display_order,
                };
                diesel::insert_into(frameworks::table)
                    .values(&new_framework)
                    .execute(conn)?;
            }
        }

        Ok(resume)
    })
    .map_err(app_err_from_diesel_err)
}
