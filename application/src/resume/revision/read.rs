use diesel::prelude::*;
use domain::models::{
    ResumeRevision, ResumeRevisionDiff, ResumeRevisionRow, ResumeRevisionSummary, ResumeSnapshot,
};
use infrastructure::establish_connection;
use uuid::Uuid;

use crate::error::ApplicationError;
use crate::resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume};
use crate::resume::revision::diff::{structural_diff, unified_diff};
use crate::resume::revision::snapshot::{load_full_resume, snapshot_from_full_resume};

/// The timeline, newest first, without snapshot bodies.
pub fn list_revisions(
    resume_id: Uuid,
    user_id: i32,
) -> Result<Vec<ResumeRevisionSummary>, ApplicationError> {
    use domain::schema::resume_revisions;

    require_owned_resume(user_id, resume_id)?;

    resume_revisions::table
        .filter(resume_revisions::resume_id.eq(resume_id))
        .select((
            resume_revisions::revision,
            resume_revisions::created_at,
            resume_revisions::created_by,
            resume_revisions::action,
            resume_revisions::is_restore,
        ))
        .order(resume_revisions::revision.desc())
        .load::<ResumeRevisionSummary>(&mut establish_connection())
        .map_err(app_err_from_diesel_err)
}

/// One revision with the state it recorded.
pub fn load_revision(
    resume_id: Uuid,
    revision: i32,
    user_id: i32,
) -> Result<ResumeRevision, ApplicationError> {
    let row = load_revision_row(resume_id, revision, user_id)?;
    let snapshot = read_snapshot(&row)?;

    Ok(ResumeRevision {
        revision: row.revision,
        created_at: row.created_at,
        created_by: row.created_by,
        action: row.action,
        is_restore: row.is_restore,
        snapshot,
    })
}

/// What the edit that produced `revision` changed: its recorded state compared
/// with the state that replaced it, which is the next revision's before-image
/// or, at the head, the resume as it stands now.
pub fn revision_diff(
    resume_id: Uuid,
    revision: i32,
    user_id: i32,
    unified: bool,
) -> Result<ResumeRevisionDiff, ApplicationError> {
    let row = load_revision_row(resume_id, revision, user_id)?;
    let before = read_snapshot(&row)?;

    let head = head_revision(resume_id)?;
    let after = if revision < head {
        let next = load_revision_row(resume_id, revision + 1, user_id)?;
        read_snapshot(&next)?
    } else {
        let mut conn = establish_connection();
        let full = load_full_resume(&mut conn, resume_id)
            .map_err(app_err_from_diesel_err)?
            .ok_or_else(|| ApplicationError::NotFound(format!("Resume with id {resume_id} not found")))?;
        snapshot_from_full_resume(&full)
    };

    Ok(ResumeRevisionDiff {
        revision,
        changes: structural_diff(&before, &after),
        unified: unified.then(|| unified_diff(&before, &after)),
    })
}

fn load_revision_row(
    resume_id: Uuid,
    revision: i32,
    user_id: i32,
) -> Result<ResumeRevisionRow, ApplicationError> {
    use domain::schema::resume_revisions;

    require_owned_resume(user_id, resume_id)?;

    first_or_404(
        "Resume revision",
        revision,
        resume_revisions::table
            .filter(resume_revisions::resume_id.eq(resume_id))
            .filter(resume_revisions::revision.eq(revision)),
    )
}

fn head_revision(resume_id: Uuid) -> Result<i32, ApplicationError> {
    use domain::schema::resume_revisions;

    let head: Option<i32> = resume_revisions::table
        .filter(resume_revisions::resume_id.eq(resume_id))
        .select(diesel::dsl::max(resume_revisions::revision))
        .first(&mut establish_connection())
        .map_err(app_err_from_diesel_err)?;

    Ok(head.unwrap_or(0))
}

fn read_snapshot(row: &ResumeRevisionRow) -> Result<ResumeSnapshot, ApplicationError> {
    ResumeSnapshot::from_json(&row.snapshot).map_err(|err| {
        ApplicationError::Internal(format!("stored revision snapshot is unreadable: {}", err))
    })
}
