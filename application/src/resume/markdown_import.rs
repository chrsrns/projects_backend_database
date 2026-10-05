use diesel::prelude::*;
use domain::models::{
    NewEducation, NewEducationKeyPoint, NewFramework, NewLanguage, NewPortfolioKeyPoint,
    NewPortfolioProject, NewPortfolioTechnology, NewResume, NewSkill, NewWorkExperience,
    NewWorkExperienceKeyPoint, PartialDate, Resume, ResumeView,
};
use infrastructure::run_in_transaction;
use shared::markdown;

use crate::{
    error::ApplicationError,
    resume::common::{
        app_err_from_diesel_err, find_resume, partial_date_to_columns, validate_executive_summary,
        validate_optional_text, validate_optional_url, validate_video,
    },
};

/// Front-matter metadata decoded into update semantics: `None` = key absent,
/// no change; `Some(None)` = empty or `null` value, store `NULL`; `Some(_)` =
/// validated write. Only a variant target may carry these.
#[derive(Default)]
struct FrontMatterUpdates {
    company_name: Option<Option<String>>,
    role_title: Option<Option<String>>,
    variant_label: Option<Option<String>>,
    job_description: Option<Option<String>>,
    target_date: Option<Option<PartialDate>>,
    show_variant_tag: Option<bool>,
}

fn front_matter_text(
    value: Option<&serde_json::Value>,
    field_name: &str,
    max_length: usize,
) -> Result<Option<Option<String>>, ApplicationError> {
    match value {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(Some(None)),
        Some(serde_json::Value::String(s)) => Ok(Some(validate_optional_text(
            Some(s.clone()),
            field_name,
            max_length,
        )?)),
        Some(other) => Err(ApplicationError::BadRequest(format!(
            "front-matter {} must be a JSON string or null, got {}",
            field_name, other
        ))),
    }
}

fn front_matter_target_date(
    value: Option<&serde_json::Value>,
) -> Result<Option<Option<PartialDate>>, ApplicationError> {
    match value {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(Some(None)),
        Some(serde_json::Value::String(s)) => PartialDate::from_iso_str(s)
            .map(|date| Some(Some(date)))
            .map_err(|err| {
                ApplicationError::BadRequest(format!(
                    "front-matter target_date is malformed: {}",
                    err
                ))
            }),
        Some(other) => Err(ApplicationError::BadRequest(format!(
            "front-matter target_date must be an ISO partial date string or null, got {}",
            other
        ))),
    }
}

fn front_matter_show_variant_tag(
    value: Option<&serde_json::Value>,
) -> Result<Option<bool>, ApplicationError> {
    match value {
        None => Ok(None),
        Some(serde_json::Value::Bool(flag)) => Ok(Some(*flag)),
        Some(other) => Err(ApplicationError::BadRequest(format!(
            "front-matter show_variant_tag must be true or false, got {}",
            other
        ))),
    }
}

/// Row update applied by a markdown import. Content fields are always set
/// (`Some(None)` stores NULL, so a dropped bullet clears the column); `email`
/// and `is_public` are skipped entirely on a variant target; the variant
/// metadata fields use the absent/null/set tri-state.
#[derive(AsChangeset)]
#[diesel(table_name = domain::schema::resumes)]
struct ImportResumeChangeset {
    name: String,
    profile_image_url: Option<Option<String>>,
    location: Option<Option<String>>,
    email: Option<String>,
    github_url: Option<Option<String>>,
    mobile_number: Option<Option<String>>,
    executive_summary: Option<Option<String>>,
    video: Option<Option<String>>,
    is_public: Option<bool>,
    company_name: Option<Option<String>>,
    role_title: Option<Option<String>>,
    variant_label: Option<Option<String>>,
    job_description: Option<Option<String>>,
    target_date: Option<Option<chrono::NaiveDate>>,
    target_date_precision: Option<Option<String>>,
    show_variant_tag: Option<bool>,
}

fn decode_front_matter_metadata(
    front_matter: &markdown::FrontMatter,
) -> Result<FrontMatterUpdates, ApplicationError> {
    Ok(FrontMatterUpdates {
        company_name: front_matter_text(front_matter.company_name.as_ref(), "Company name", 255)?,
        role_title: front_matter_text(front_matter.role_title.as_ref(), "Role title", 255)?,
        variant_label: front_matter_text(
            front_matter.variant_label.as_ref(),
            "Variant label",
            255,
        )?,
        job_description: front_matter_text(
            front_matter.job_description.as_ref(),
            "Job description",
            20_000,
        )?,
        target_date: front_matter_target_date(front_matter.target_date.as_ref())?,
        show_variant_tag: front_matter_show_variant_tag(front_matter.show_variant_tag.as_ref())?,
    })
}

/// Decodes the `resume_id` routing marker. The marker must be a positive
/// integer, either as a JSON number or a numeric JSON string; an empty value
/// or any other scalar is malformed.
fn resume_id_marker(value: &serde_json::Value) -> Result<i32, ApplicationError> {
    let bad = || {
        ApplicationError::BadRequest(
            "front-matter resume_id must be a positive integer".to_string(),
        )
    };
    match value {
        serde_json::Value::Number(n) => n
            .as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .filter(|v| *v > 0)
            .ok_or_else(bad),
        serde_json::Value::String(s) => s
            .trim()
            .parse::<i32>()
            .ok()
            .filter(|v| *v > 0)
            .ok_or_else(bad),
        _ => Err(bad()),
    }
}

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

/// Imports a markdown resume. `explicit_id` is the URL target of
/// `POST /api/resume/{id}/import/markdown`; its ownership check runs before
/// any front-matter handling and a front-matter `resume_id` must match it.
/// Otherwise the front-matter `resume_id` marker picks the target, with the
/// email match as the marker-less fallback and creation as the last resort.
pub fn import_resume_markdown(
    markdown: &str,
    user_id_value: i32,
    explicit_id: Option<i32>,
) -> Result<(ResumeView, bool), ApplicationError> {
    let explicit_target = match explicit_id {
        Some(resume_id) => {
            let row = find_resume(resume_id)?;
            match row.created_by {
                Some(owner) if owner == user_id_value => {}
                _ => return Err(ApplicationError::Forbidden),
            }
            Some(row)
        }
        None => None,
    };

    let parsed = markdown::parse_resume_markdown(markdown).map_err(|err| match err {
        markdown::MarkdownError::InvalidMarkdown(msg) => ApplicationError::BadRequest(msg),
    })?;
    let front_matter = parsed.front_matter.unwrap_or_default();
    let full_resume = parsed.resume;

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

    let executive_summary = validate_executive_summary(full_resume.executive_summary.clone())?;
    let video = validate_video(full_resume.video.clone())?;
    let portfolio_video_urls: Vec<Option<String>> = full_resume
        .portfolio_projects
        .iter()
        .map(|p| validate_optional_url(p.video_url.clone(), "Video URL"))
        .collect::<Result<Vec<_>, _>>()?;
    let portfolio_image_urls: Vec<Option<String>> = full_resume
        .portfolio_projects
        .iter()
        .map(|p| validate_optional_url(p.image_url.clone(), "Image URL"))
        .collect::<Result<Vec<_>, _>>()?;
    let portfolio_project_links: Vec<Option<String>> = full_resume
        .portfolio_projects
        .iter()
        .map(|p| validate_optional_url(p.project_link.clone(), "Project Link"))
        .collect::<Result<Vec<_>, _>>()?;
    let portfolio_source_code_links: Vec<Option<String>> = full_resume
        .portfolio_projects
        .iter()
        .map(|p| validate_optional_url(p.source_code_link.clone(), "Source Code Link"))
        .collect::<Result<Vec<_>, _>>()?;

    let marker_id = front_matter
        .resume_id
        .as_ref()
        .map(resume_id_marker)
        .transpose()?;

    if let (Some(target_id), Some(marker_id)) = (explicit_id, marker_id)
        && marker_id != target_id
    {
        return Err(ApplicationError::BadRequest(
            "front-matter resume_id does not match the target resume id".to_string(),
        ));
    }

    let mut conn = infrastructure::establish_connection();

    // Resolution order: an explicit route id wins, then the front-matter
    // marker, then the email match (base rows only), then creation. A marker
    // that names a missing or foreign row fails loudly instead of falling
    // back to the email match.
    let target_row: Option<Resume> = match explicit_target {
        Some(row) => Some(row),
        None => match marker_id {
            Some(marker_id) => {
                use domain::schema::resumes;
                let row = resumes::table
                    .find(marker_id)
                    .first::<Resume>(&mut conn)
                    .optional()
                    .map_err(app_err_from_diesel_err)?;
                match row {
                    Some(r) if r.created_by == Some(user_id_value) => Some(r),
                    Some(_) => return Err(ApplicationError::Forbidden),
                    None => {
                        return Err(ApplicationError::NotFound(format!(
                            "Resume with id {} not found",
                            marker_id
                        )));
                    }
                }
            }
            None => {
                use domain::schema::resumes;
                use domain::schema::resumes::dsl::*;
                // A variant carries its base's email, so the match must ignore
                // variant rows or an import would silently rewrite a tailored copy
                // instead of the base resume it belongs to.
                let existing = resumes::table
                    .filter(email.eq(&full_resume.email))
                    .filter(base_resume_id.is_null())
                    .first::<Resume>(&mut conn)
                    .optional()
                    .map_err(app_err_from_diesel_err)?;
                match existing {
                    Some(r) if r.created_by == Some(user_id_value) => Some(r),
                    Some(_) => return Err(ApplicationError::Forbidden),
                    None => None,
                }
            }
        },
    };

    let target_is_variant = target_row
        .as_ref()
        .is_some_and(|r| r.base_resume_id.is_some());

    // Metadata keys exist to retarget variants; on a base target or the
    // create path there is nothing to apply them to.
    if !target_is_variant && front_matter.has_metadata() {
        return Err(ApplicationError::BadRequest(
            "front-matter metadata keys require a variant import target".to_string(),
        ));
    }

    let metadata = if target_is_variant {
        decode_front_matter_metadata(&front_matter)?
    } else {
        FrontMatterUpdates::default()
    };

    let existing_id = target_row.as_ref().map(|r| r.id);

    let new_resume = NewResume {
        name: full_resume.name.clone(),
        profile_image_url: full_resume.profile_image_url.clone(),
        location: full_resume.location.clone(),
        email: full_resume.email.clone(),
        github_url: full_resume.github_url.clone(),
        mobile_number: full_resume.mobile_number.clone(),
        executive_summary,
        video,
        created_by: Some(user_id_value),
        is_public: full_resume.is_public,
        base_resume_id: None,
        company_name: None,
        role_title: None,
        target_date: None,
        target_date_precision: None,
        job_description: None,
        variant_label: None,
        show_variant_tag: true,
    };

    run_in_transaction(&mut conn, move |conn| {
        use domain::schema::{
            education, education_key_points, frameworks, languages, portfolio_key_points,
            portfolio_projects, portfolio_technologies, resumes, skills,
            work_experience_key_points, work_experiences,
        };

        let resume = if let Some(resume_id) = existing_id {
            diesel::delete(skills::table.filter(skills::dsl::resume_id.eq(resume_id)))
                .execute(conn)?;
            diesel::delete(languages::table.filter(languages::dsl::resume_id.eq(resume_id)))
                .execute(conn)?;
            diesel::delete(education::table.filter(education::dsl::resume_id.eq(resume_id)))
                .execute(conn)?;
            diesel::delete(
                work_experiences::table.filter(work_experiences::dsl::resume_id.eq(resume_id)),
            )
            .execute(conn)?;
            diesel::delete(
                portfolio_projects::table.filter(portfolio_projects::dsl::resume_id.eq(resume_id)),
            )
            .execute(conn)?;

            // A variant keeps its stored email, is_public and created_by:
            // the markdown Email/Public bullets stay required by the format
            // but are ignored on a variant target. Front-matter metadata
            // keys apply with absent = no change and empty/null = NULL.
            let changeset = ImportResumeChangeset {
                name: new_resume.name.clone(),
                profile_image_url: Some(new_resume.profile_image_url.clone()),
                location: Some(new_resume.location.clone()),
                email: if target_is_variant {
                    None
                } else {
                    Some(new_resume.email.clone())
                },
                github_url: Some(new_resume.github_url.clone()),
                mobile_number: Some(new_resume.mobile_number.clone()),
                executive_summary: Some(new_resume.executive_summary.clone()),
                video: Some(new_resume.video.clone()),
                is_public: if target_is_variant {
                    None
                } else {
                    Some(new_resume.is_public)
                },
                company_name: metadata.company_name.clone(),
                role_title: metadata.role_title.clone(),
                variant_label: metadata.variant_label.clone(),
                job_description: metadata.job_description.clone(),
                target_date: metadata
                    .target_date
                    .map(|value| value.map(|date| partial_date_to_columns(date).0)),
                target_date_precision: metadata
                    .target_date
                    .map(|value| value.map(|date| partial_date_to_columns(date).1)),
                show_variant_tag: metadata.show_variant_tag,
            };
            diesel::update(resumes::table.find(resume_id))
                .set(&changeset)
                .get_result::<Resume>(conn)?
        } else {
            diesel::insert_into(resumes::table)
                .values(&new_resume)
                .get_result::<Resume>(conn)?
        };

        let mut education_id_map: Vec<i32> = Vec::new();
        for (idx, edu) in full_resume.education.iter().enumerate() {
            let end_pair = edu
                .end_date
                .as_ref()
                .map(|end| (end.canonical_end_date(), end.precision.to_string()));
            let new_edu = NewEducation {
                resume_id: resume.id,
                education_stage: edu.education_stage.clone(),
                institution_name: edu.institution_name.clone(),
                degree: edu.degree.clone(),
                start_date: edu.start_date.canonical_start_date(),
                start_date_precision: edu.start_date.precision.to_string(),
                end_date: end_pair.as_ref().map(|(d, _)| *d),
                end_date_precision: end_pair.map(|(_, p)| p),
                description: edu.description.clone(),
                display_order: edu.display_order.or(Some(idx as i32)),
                active: true,
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
                active: true,
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
            let end_pair = work
                .end_date
                .as_ref()
                .map(|end| (end.canonical_end_date(), end.precision.to_string()));
            let new_work = NewWorkExperience {
                resume_id: resume.id,
                job_title: work.job_title.clone(),
                company_name: work.company_name.clone(),
                start_date: work.start_date.canonical_start_date(),
                start_date_precision: work.start_date.precision.to_string(),
                end_date: end_pair.as_ref().map(|(d, _)| *d),
                end_date_precision: end_pair.map(|(_, p)| p),
                description: work.description.clone(),
                display_order: work.display_order.or(Some(idx as i32)),
                active: true,
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
                active: true,
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
                image_url: portfolio_image_urls[idx].clone(),
                project_link: portfolio_project_links[idx].clone(),
                source_code_link: portfolio_source_code_links[idx].clone(),
                video_url: portfolio_video_urls[idx].clone(),
                description: project.description.clone(),
                display_order: project.display_order.or(Some(idx as i32)),
                active: true,
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
                active: true,
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
                active: true,
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

        Ok((resume, existing_id.is_none()))
    })
    // The email match only ever resolves a base resume, but the marker and
    // the explicit route may target a variant; its base is owned by the same
    // user, so it is always reachable for this viewer.
    .map(|(resume, created)| {
        let base_accessible = resume.base_resume_id.is_some();
        (
            ResumeView::from_resume(resume, base_accessible, true),
            created,
        )
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
