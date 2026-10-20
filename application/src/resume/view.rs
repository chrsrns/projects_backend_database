use std::collections::HashMap;

use diesel::alias;
use diesel::expression::BoxableExpression;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text};
use domain::models::{
    Resume, ResumeGroup, ResumeListOrder, ResumeListQuery, ResumeListSort, ResumePage, ResumeView,
    Visibility,
};
use domain::schema::resumes;
use infrastructure::establish_connection;

use crate::error::ApplicationError;
use crate::resume::common::{app_err_from_diesel_err, viewer_is_owner};
use uuid::Uuid;

alias!(resumes as base_resume: BaseResume, resumes as joined_resume: JoinedResume);

/// Maps each resume id to whether the viewer may read the base resume it
/// points at. Resolved with a single self-join for the whole page rather
/// than one lookup per row.
fn base_accessibility(
    viewer: Option<i32>,
    resume_ids: &[Uuid],
) -> Result<HashMap<Uuid, bool>, ApplicationError> {
    if resume_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<(Option<Uuid>, String, i32)> = joined_resume
        .inner_join(
            base_resume.on(base_resume
                .field(resumes::id)
                .nullable()
                .eq(joined_resume.field(resumes::base_resume_id))),
        )
        .filter(joined_resume.field(resumes::id).eq_any(resume_ids))
        .select((
            joined_resume.field(resumes::base_resume_id),
            base_resume.field(resumes::visibility),
            base_resume.field(resumes::created_by),
        ))
        .load(&mut establish_connection())
        .map_err(app_err_from_diesel_err)?;

    let mut accessibility = HashMap::new();
    for (base_id, base_visibility, base_owner) in rows {
        if let Some(base_id) = base_id {
            let reachable =
                base_visibility == Visibility::Public.as_str() || viewer == Some(base_owner);
            accessibility.insert(base_id, reachable);
        }
    }

    Ok(accessibility)
}

/// By-id read access (V218): `unlisted` and `public` are reachable by
/// anyone who knows the UUID; `private` only for the owner.
fn accessible_resume(
    viewer: Option<i32>,
    resume_id: Uuid,
) -> Result<Vec<Resume>, ApplicationError> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut query = resumes_dsl::resumes
        .filter(resumes_dsl::id.eq(resume_id))
        .into_boxed();
    query = match viewer {
        Some(user_id) => query.filter(
            resumes_dsl::visibility
                .ne(Visibility::Private.as_str())
                .or(resumes_dsl::created_by.eq(user_id)),
        ),
        None => query.filter(resumes_dsl::visibility.ne(Visibility::Private.as_str())),
    };

    query
        .select(resumes::all_columns)
        .load::<Resume>(&mut establish_connection())
        .map_err(app_err_from_diesel_err)
}

/// LIKE metacharacters are literal in a search term (V256).
fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// The V252 term predicate on the outer (unit) row: every field is an
/// ILIKE substring match, `target_date` cast to text for the operator.
fn term_predicate(
    term: &str,
) -> Box<dyn BoxableExpression<resumes::table, Pg, SqlType = Nullable<Bool>>> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let pattern = format!("%{}%", escape_like(term));
    Box::new(
        resumes_dsl::name
            .ilike(pattern.clone())
            .or(resumes_dsl::email.ilike(pattern.clone()))
            .or(resumes_dsl::company_name.ilike(pattern.clone()))
            .or(resumes_dsl::role_title.ilike(pattern.clone()))
            .or(resumes_dsl::variant_label.ilike(pattern.clone()))
            .or(resumes_dsl::job_description.ilike(pattern.clone()))
            .or(resumes_dsl::target_date
                .cast::<Nullable<Text>>()
                .ilike(pattern)),
    )
}

/// Listing access (V211) + the grouped page-unit set (V257) + the search
/// term set (V252/V260): non-owners see `public` rows only; `unlisted` is
/// never listed. A unit is a scoped row that is not a variant of a
/// caller-visible base — the exclusion predicate lives in the page query
/// itself. Search qualifies a unit when its own row matches every term OR
/// a single caller-visible variant matches every term — the variant side
/// arrives as `matching_base_ids`, the V211-scoped set of bases owning a
/// variant matching the whole term set (row-local per row of the
/// pre-query, scope-checked per V260).
fn scoped_units(
    viewer: Option<i32>,
    query: &ResumeListQuery,
    visible_base_ids: Vec<Uuid>,
    matching_base_ids: Vec<Uuid>,
) -> resumes::BoxedQuery<'static, Pg> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut q = resumes_dsl::resumes.into_boxed();
    q = q.filter(
        resumes_dsl::visibility
            .eq(Visibility::Public.as_str())
            .or(resumes_dsl::created_by.eq_any(viewer)),
    );
    q = q.filter(resumes_dsl::base_resume_id.is_null().or(diesel::dsl::not(
        resumes_dsl::base_resume_id.eq_any(visible_base_ids),
    )));

    if !query.search_terms.is_empty() {
        let mut own: Option<
            Box<dyn BoxableExpression<resumes::table, Pg, SqlType = Nullable<Bool>>>,
        > = None;
        for term in &query.search_terms {
            let pred = term_predicate(term);
            own = Some(match own {
                None => pred,
                Some(prev) => Box::new(prev.and(pred)),
            });
        }
        q = q.filter(
            own.expect("non-empty term set")
                .or(resumes_dsl::id.eq_any(matching_base_ids)),
        );
    }

    q
}

fn listed_resumes(
    viewer: Option<i32>,
    query: &ResumeListQuery,
) -> Result<ResumePage, ApplicationError> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut conn = establish_connection();

    // Bases in the viewer's V211 scope — the set a variant's base must be
    // in for the variant to attach instead of standing alone.
    let visible_base_ids: Vec<Uuid> = resumes_dsl::resumes
        .filter(
            resumes_dsl::visibility
                .eq(Visibility::Public.as_str())
                .or(resumes_dsl::created_by.eq_any(viewer)),
        )
        .select(resumes_dsl::id)
        .load(&mut conn)
        .map_err(app_err_from_diesel_err)?;

    // V260: bases owning a caller-visible variant that matches every
    // search term on that one row. Empty unless a search is present.
    let mut matching_base_ids: Vec<Uuid> = Vec::new();
    if !query.search_terms.is_empty() {
        let mut mq = resumes_dsl::resumes.into_boxed();
        mq = mq.filter(resumes_dsl::base_resume_id.is_not_null()).filter(
            resumes_dsl::visibility
                .eq(Visibility::Public.as_str())
                .or(resumes_dsl::created_by.eq_any(viewer)),
        );
        for term in &query.search_terms {
            mq = mq.filter(term_predicate(term));
        }
        matching_base_ids = mq
            .select(resumes_dsl::base_resume_id)
            .distinct()
            .load::<Option<Uuid>>(&mut conn)
            .map_err(app_err_from_diesel_err)?
            .into_iter()
            .flatten()
            .collect();
    }

    let total: i64 = scoped_units(
        viewer,
        query,
        visible_base_ids.clone(),
        matching_base_ids.clone(),
    )
    .count()
    .get_result(&mut conn)
    .map_err(app_err_from_diesel_err)?;

    let mut q = scoped_units(viewer, query, visible_base_ids, matching_base_ids);
    q = match (query.sort, query.order) {
        (ResumeListSort::Id, ResumeListOrder::Asc) => q.order(resumes_dsl::id.asc()),
        (ResumeListSort::Id, ResumeListOrder::Desc) => q.order(resumes_dsl::id.desc()),
        (ResumeListSort::Name, ResumeListOrder::Asc) => q
            .order(resumes_dsl::name.asc())
            .then_order_by(resumes_dsl::id.asc()),
        (ResumeListSort::Name, ResumeListOrder::Desc) => q
            .order(resumes_dsl::name.desc())
            .then_order_by(resumes_dsl::id.asc()),
        (ResumeListSort::UpdatedAt, ResumeListOrder::Asc) => q
            .order(resumes_dsl::updated_at.asc())
            .then_order_by(resumes_dsl::id.asc()),
        (ResumeListSort::UpdatedAt, ResumeListOrder::Desc) => q
            .order(resumes_dsl::updated_at.desc())
            .then_order_by(resumes_dsl::id.asc()),
    };

    let rows = q
        .select(resumes::all_columns)
        .limit(query.limit)
        .offset(query.offset)
        .load::<Resume>(&mut conn)
        .map_err(app_err_from_diesel_err)?;

    // V258: one batched variant fetch for the whole page — every
    // caller-visible variant of every unit's base, ordered `id` ascending.
    let base_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let mut variant_query = resumes_dsl::resumes
        .filter(resumes_dsl::base_resume_id.eq_any(&base_ids))
        .into_boxed();
    variant_query = variant_query.filter(
        resumes_dsl::visibility
            .eq(Visibility::Public.as_str())
            .or(resumes_dsl::created_by.eq_any(viewer)),
    );
    let variant_rows = variant_query
        .order(resumes_dsl::id.asc())
        .select(resumes::all_columns)
        .load::<Resume>(&mut conn)
        .map_err(app_err_from_diesel_err)?;

    let mut variants_by_base: HashMap<Uuid, Vec<ResumeView>> = HashMap::new();
    for variant in build_views(viewer, variant_rows)? {
        if let Some(base_id) = variant.base_resume_id {
            variants_by_base.entry(base_id).or_default().push(variant);
        }
    }

    let items = build_views(viewer, rows)?
        .into_iter()
        .map(|base| {
            let variants = variants_by_base.remove(&base.id).unwrap_or_default();
            ResumeGroup { base, variants }
        })
        .collect();

    Ok(ResumePage {
        items,
        total,
        limit: query.limit,
        offset: query.offset,
    })
}

fn build_views(
    viewer: Option<i32>,
    items: Vec<Resume>,
) -> Result<Vec<ResumeView>, ApplicationError> {
    let resume_ids: Vec<Uuid> = items.iter().map(|resume| resume.id).collect();
    let accessibility = base_accessibility(viewer, &resume_ids)?;

    Ok(items
        .into_iter()
        .map(|resume| {
            let is_owner = viewer_is_owner(&resume, viewer);
            let base_reachable = resume
                .base_resume_id
                .and_then(|base_id| accessibility.get(&base_id).copied())
                .unwrap_or(false);
            ResumeView::from_resume(resume, base_reachable, is_owner)
        })
        .collect())
}

pub fn load_resume_view(
    resume_id: Uuid,
    viewer: Option<i32>,
) -> Result<ResumeView, ApplicationError> {
    let mut items = accessible_resume(viewer, resume_id)?;

    match items.pop() {
        Some(resume) => Ok(build_views(viewer, vec![resume])?
            .pop()
            .expect("a single resume builds a single view")),
        None => Err(ApplicationError::NotFound(format!(
            "Resume with id {} not found",
            resume_id
        ))),
    }
}

pub fn load_resume_views(
    viewer: Option<i32>,
    query: &ResumeListQuery,
) -> Result<ResumePage, ApplicationError> {
    listed_resumes(viewer, query)
}

/// Builds views for rows the caller has already proven they may read,
/// such as a freshly created resume or the variants of an accessible
/// base. `base_linkage_visible` is the V218 linkage-reveal answer for the
/// shared base (base `public` or owned by the viewer), not by-id access.
pub fn views_for_accessible_base(
    items: Vec<Resume>,
    viewer: Option<i32>,
    base_linkage_visible: bool,
) -> Vec<ResumeView> {
    items
        .into_iter()
        .map(|resume| {
            let is_owner = viewer_is_owner(&resume, viewer);
            ResumeView::from_resume(resume, base_linkage_visible, is_owner)
        })
        .collect()
}
