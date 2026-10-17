use diesel::prelude::*;
use domain::models::{NewResumeRevision, Resume};
use infrastructure::DbConnection;
use uuid::Uuid;

use crate::realtime::ResumeChangedAction;
use crate::resume::revision::snapshot::{load_full_resume, snapshot_from_full_resume};

/// Records the aggregate as it stands as the next revision for `resume_id`.
///
/// Must be called inside the transaction that performs the mutation and before
/// it writes anything: the snapshot is the state the edit replaces. The resume
/// row is locked first, so the pre-state read and the revision number are
/// serialized against every other writer of the same resume.
///
/// Returns the revision number written.
pub fn capture(
    conn: &mut DbConnection,
    resume_id: Uuid,
    action: &str,
    is_restore: bool,
) -> QueryResult<i32> {
    use domain::schema::{resume_revisions, resumes};

    let resume = resumes::table
        .find(resume_id)
        .for_update()
        .first::<Resume>(conn)
        .optional()?;

    let Some(resume) = resume else {
        return Err(diesel::result::Error::NotFound);
    };

    // Every mutating use case is owner gated, so the row always has an author.
    // A missing one is an internal inconsistency, not a client error.
    let Some(created_by) = resume.created_by else {
        return Err(diesel::result::Error::RollbackTransaction);
    };

    let full = load_full_resume(conn, resume_id)?.ok_or(diesel::result::Error::NotFound)?;
    let snapshot = snapshot_from_full_resume(&full)
        .to_json()
        .map_err(|err| diesel::result::Error::SerializationError(err.to_string().into()))?;

    let head: Option<i32> = resume_revisions::table
        .filter(resume_revisions::resume_id.eq(resume_id))
        .select(diesel::dsl::max(resume_revisions::revision))
        .first(conn)?;

    let revision = head.unwrap_or(0) + 1;

    diesel::insert_into(resume_revisions::table)
        .values(&NewResumeRevision {
            resume_id,
            revision,
            created_by,
            action: action.to_string(),
            is_restore,
            snapshot,
        })
        .execute(conn)?;

    Ok(revision)
}

/// Captures for a change action, or does nothing for the actions that must not
/// capture. A create has no before-image and a resume delete takes its trail
/// with it.
pub fn capture_for_action(
    conn: &mut DbConnection,
    resume_id: Uuid,
    action: &ResumeChangedAction,
) -> QueryResult<()> {
    match action {
        ResumeChangedAction::Updated(section) => {
            capture(conn, resume_id, section.as_str(), false).map(|_| ())
        }
        ResumeChangedAction::Created | ResumeChangedAction::Deleted => Ok(()),
    }
}
