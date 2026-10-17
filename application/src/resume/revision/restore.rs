use chrono::NaiveDate;
use diesel::prelude::*;
use domain::models::{
    NewEducation, NewEducationKeyPoint, NewFramework, NewLanguage, NewPortfolioKeyPoint,
    NewPortfolioProject, NewPortfolioTechnology, NewSkill, NewWorkExperience,
    NewWorkExperienceKeyPoint, Resume, ResumeSnapshot, ResumeView,
};
use infrastructure::{DbConnection, establish_connection};
use uuid::Uuid;

use crate::error::ApplicationError;
use crate::realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType};
use crate::resume::common::{app_err_from_diesel_err, require_owned_resume};
use crate::resume::revision::capture::capture;
use crate::resume::revision::read::head_revision;

/// Restores the state a revision recorded and appends the result as the next
/// revision, so the trail stays complete: the state the restore replaced is
/// what the appended revision holds.
///
/// The head read, the guard, the capture and the append all run under one lock
/// on the resume row, otherwise a concurrent edit can commit between the guard
/// check and the append.
pub fn restore_revision(
    user_id: i32,
    resume_id: Uuid,
    revision: i32,
    expected_revision: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<ResumeView, ApplicationError> {
    require_owned_resume(user_id, resume_id)?;

    let mut conn = establish_connection();

    let restored = conn
        .transaction::<Resume, RestoreError, _>(|conn| {
            let resume = resume_row(conn, resume_id)?;

            let head = head_revision(conn, resume_id)?;
            if revision < 1 || revision > head {
                return Err(RestoreError::App(revision_not_found(revision)));
            }

            if expected_revision != head {
                return Err(RestoreError::App(ApplicationError::Conflict(format!(
                    "Resume head is revision {head}, not {expected_revision}"
                ))));
            }

            let snapshot = snapshot_at(conn, resume_id, revision)?;

            capture(conn, resume_id, SectionType::PersonalInfo.as_str(), true)?;

            apply_snapshot(conn, &resume, &snapshot)?;

            use domain::schema::resumes;

            resumes::table
                .find(resume_id)
                .first::<Resume>(conn)
                .map_err(RestoreError::Db)
        })
        .map_err(|err| match err {
            RestoreError::App(err) => err,
            RestoreError::Db(err) => app_err_from_diesel_err(err),
        })?;

    publisher.resume_changed(
        resume_id,
        ResumeChangedAction::Updated(SectionType::PersonalInfo),
    );

    Ok(ResumeView::from_resume(restored, true, true))
}

enum RestoreError {
    App(ApplicationError),
    Db(diesel::result::Error),
}

impl From<diesel::result::Error> for RestoreError {
    fn from(err: diesel::result::Error) -> Self {
        RestoreError::Db(err)
    }
}

fn revision_not_found(revision: i32) -> ApplicationError {
    ApplicationError::NotFound(format!("Resume revision with id {revision} not found"))
}

fn resume_row(conn: &mut DbConnection, resume_id: Uuid) -> Result<Resume, RestoreError> {
    use domain::schema::resumes;

    resumes::table
        .find(resume_id)
        .for_update()
        .first::<Resume>(conn)
        .optional()
        .map_err(RestoreError::Db)?
        .ok_or_else(|| {
            RestoreError::App(ApplicationError::NotFound(format!(
                "Resume with id {resume_id} not found"
            )))
        })
}

fn snapshot_at(
    conn: &mut DbConnection,
    resume_id: Uuid,
    revision: i32,
) -> Result<ResumeSnapshot, RestoreError> {
    use domain::schema::resume_revisions;

    let snapshot: Option<serde_json::Value> = resume_revisions::table
        .filter(resume_revisions::resume_id.eq(resume_id))
        .filter(resume_revisions::revision.eq(revision))
        .select(resume_revisions::snapshot)
        .first(conn)
        .optional()
        .map_err(RestoreError::Db)?;

    let snapshot = snapshot.ok_or_else(|| RestoreError::App(revision_not_found(revision)))?;

    ResumeSnapshot::from_json(&snapshot).map_err(|err| {
        RestoreError::App(ApplicationError::Internal(format!(
            "stored revision snapshot is unreadable: {}",
            err
        )))
    })
}

/// What a restore writes back: every content column, with `email` left alone
/// for a variant because a variant's email is immutable after the clone.
#[derive(AsChangeset)]
#[diesel(table_name = domain::schema::resumes)]
struct RestoreChangeset {
    name: String,
    profile_image_url: Option<String>,
    location: Option<String>,
    email: Option<String>,
    github_url: Option<String>,
    mobile_number: Option<String>,
    executive_summary: Option<String>,
    video: Option<String>,
    visibility: String,
    company_name: Option<String>,
    role_title: Option<String>,
    variant_label: Option<String>,
    job_description: Option<String>,
    target_date: Option<NaiveDate>,
    target_date_precision: Option<String>,
    show_variant_tag: bool,
}

fn apply_snapshot(
    conn: &mut DbConnection,
    resume: &Resume,
    snapshot: &ResumeSnapshot,
) -> Result<(), diesel::result::Error> {
    use domain::schema::{
        education, education_key_points, frameworks, languages, portfolio_key_points,
        portfolio_projects, portfolio_technologies, resumes, skills, work_experience_key_points,
        work_experiences,
    };

    let resume_id = resume.id;
    let is_variant = resume.base_resume_id.is_some();

    // Children are replaced wholesale: the snapshot is the complete state, and
    // its child ids are the ids the rows carried when it was captured, not
    // identities that survive the restore.
    diesel::delete(skills::table.filter(skills::dsl::resume_id.eq(resume_id))).execute(conn)?;
    diesel::delete(languages::table.filter(languages::dsl::resume_id.eq(resume_id))).execute(conn)?;
    diesel::delete(education::table.filter(education::dsl::resume_id.eq(resume_id))).execute(conn)?;
    diesel::delete(
        work_experiences::table.filter(work_experiences::dsl::resume_id.eq(resume_id)),
    )
    .execute(conn)?;
    diesel::delete(
        portfolio_projects::table.filter(portfolio_projects::dsl::resume_id.eq(resume_id)),
    )
    .execute(conn)?;

    let target_date = snapshot.target_date.map(|date| date.canonical_start_date());
    let target_date_precision = snapshot
        .target_date
        .map(|date| date.precision.to_string())
        .or_else(|| snapshot.target_date_precision.clone());

    diesel::update(resumes::table.find(resume_id))
        .set(&RestoreChangeset {
            name: snapshot.name.clone(),
            profile_image_url: snapshot.profile_image_url.clone(),
            location: snapshot.location.clone(),
            email: if is_variant {
                None
            } else {
                Some(snapshot.email.clone())
            },
            github_url: snapshot.github_url.clone(),
            mobile_number: snapshot.mobile_number.clone(),
            executive_summary: snapshot.executive_summary.clone(),
            video: snapshot.video.clone(),
            visibility: snapshot.visibility.as_str().to_string(),
            company_name: snapshot.company_name.clone(),
            role_title: snapshot.role_title.clone(),
            variant_label: snapshot.variant_label.clone(),
            job_description: snapshot.job_description.clone(),
            target_date,
            target_date_precision,
            show_variant_tag: snapshot.show_variant_tag,
        })
        .execute(conn)?;

    for item in &snapshot.education {
        let end_pair = item
            .end_date
            .map(|end| (end.canonical_end_date(), end.precision.to_string()));

        let inserted: domain::models::Education = diesel::insert_into(education::table)
            .values(&NewEducation {
                resume_id,
                education_stage: item.education_stage.clone(),
                institution_name: item.institution_name.clone(),
                degree: item.degree.clone(),
                start_date: item.start_date.canonical_start_date(),
                start_date_precision: item.start_date.precision.to_string(),
                end_date: end_pair.as_ref().map(|(date, _)| *date),
                end_date_precision: end_pair.as_ref().map(|(_, precision)| precision.clone()),
                description: item.description.clone(),
                display_order: item.display_order,
                active: item.active,
            })
            .get_result(conn)?;

        for key_point in snapshot
            .education_key_points
            .get(&item.id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            diesel::insert_into(education_key_points::table)
                .values(&NewEducationKeyPoint {
                    education_id: inserted.id,
                    key_point: key_point.key_point.clone(),
                    display_order: key_point.display_order,
                    active: key_point.active,
                })
                .execute(conn)?;
        }
    }

    for item in &snapshot.skills {
        diesel::insert_into(skills::table)
            .values(&NewSkill {
                resume_id,
                skill_name: item.skill_name.clone(),
                confidence_percentage: item.confidence_percentage,
                display_order: item.display_order,
            })
            .execute(conn)?;
    }

    for item in &snapshot.work_experiences {
        let end_pair = item
            .end_date
            .map(|end| (end.canonical_end_date(), end.precision.to_string()));

        let inserted: domain::models::WorkExperience = diesel::insert_into(work_experiences::table)
            .values(&NewWorkExperience {
                resume_id,
                job_title: item.job_title.clone(),
                company_name: item.company_name.clone(),
                start_date: item.start_date.canonical_start_date(),
                start_date_precision: item.start_date.precision.to_string(),
                end_date: end_pair.as_ref().map(|(date, _)| *date),
                end_date_precision: end_pair.as_ref().map(|(_, precision)| precision.clone()),
                description: item.description.clone(),
                display_order: item.display_order,
                active: item.active,
            })
            .get_result(conn)?;

        for key_point in snapshot
            .work_experience_key_points
            .get(&item.id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            diesel::insert_into(work_experience_key_points::table)
                .values(&NewWorkExperienceKeyPoint {
                    work_experience_id: inserted.id,
                    key_point: key_point.key_point.clone(),
                    display_order: key_point.display_order,
                    active: key_point.active,
                })
                .execute(conn)?;
        }
    }

    for item in &snapshot.portfolio_projects {
        let inserted: domain::models::PortfolioProject =
            diesel::insert_into(portfolio_projects::table)
                .values(&NewPortfolioProject {
                    resume_id,
                    project_name: item.project_name.clone(),
                    image_url: item.image_url.clone(),
                    project_link: item.project_link.clone(),
                    source_code_link: item.source_code_link.clone(),
                    video_url: item.video_url.clone(),
                    description: item.description.clone(),
                    display_order: item.display_order,
                    active: item.active,
                })
                .get_result(conn)?;

        for key_point in snapshot
            .portfolio_key_points
            .get(&item.id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            diesel::insert_into(portfolio_key_points::table)
                .values(&NewPortfolioKeyPoint {
                    portfolio_project_id: inserted.id,
                    key_point: key_point.key_point.clone(),
                    display_order: key_point.display_order,
                    active: key_point.active,
                })
                .execute(conn)?;
        }

        for technology in snapshot
            .portfolio_technologies
            .get(&item.id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            diesel::insert_into(portfolio_technologies::table)
                .values(&NewPortfolioTechnology {
                    portfolio_project_id: inserted.id,
                    technology_name: technology.technology_name.clone(),
                    display_order: technology.display_order,
                    active: technology.active,
                })
                .execute(conn)?;
        }
    }

    for item in &snapshot.languages {
        let inserted: domain::models::Language = diesel::insert_into(languages::table)
            .values(&NewLanguage {
                resume_id,
                language_name: item.language_name.clone(),
                display_order: item.display_order,
            })
            .get_result(conn)?;

        for framework in snapshot
            .frameworks
            .get(&item.id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            diesel::insert_into(frameworks::table)
                .values(&NewFramework {
                    language_id: inserted.id,
                    framework_name: framework.framework_name.clone(),
                    display_order: framework.display_order,
                })
                .execute(conn)?;
        }
    }

    Ok(())
}
