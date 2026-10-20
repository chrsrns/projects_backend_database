use rocket::form::FromForm;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::ResumeView;

/// Paged listing response: `items` is the requested window, `total` the
/// size of the caller-visible set before pagination, `limit`/`offset` echo
/// the applied values.
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumePage {
    pub items: Vec<ResumeGroup>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// One page unit: `base` is the unit's own row — a standalone variant when
/// its base is caller-invisible — and `variants` holds every caller-visible
/// variant of that base, ordered `id` ascending.
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResumeGroup {
    pub base: ResumeView,
    pub variants: Vec<ResumeView>,
}

/// Raw query capture for `GET /resumes`. `Vec<String>` fields collect
/// repeated keys so a duplicated parameter is visible to validation
/// instead of collapsing; `strict` surfaces unknown keys as form errors.
/// The route binds `Result<_, Errors>` so a bad query answers 400 instead
/// of silently defaulting.
#[derive(FromForm)]
#[form(strict)]
pub struct ResumeListParams {
    pub limit: Vec<String>,
    pub offset: Vec<String>,
    pub sort: Vec<String>,
    pub order: Vec<String>,
    pub search: Vec<String>,
}

pub const RESUME_LIST_DEFAULT_LIMIT: i64 = 20;
pub const RESUME_LIST_MAX_LIMIT: i64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeListSort {
    Id,
    Name,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeListOrder {
    Asc,
    Desc,
}

/// Validated listing query. `search_terms` is the whitespace-split AND set;
/// an empty vec means no filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeListQuery {
    pub limit: i64,
    pub offset: i64,
    pub sort: ResumeListSort,
    pub order: ResumeListOrder,
    pub search_terms: Vec<String>,
}

fn single<'a>(values: &'a [String], name: &str) -> Result<Option<&'a str>, String> {
    match values.len() {
        0 => Ok(None),
        1 => Ok(Some(values[0].as_str())),
        _ => Err(format!("duplicate parameter: {name}")),
    }
}

impl TryFrom<ResumeListParams> for ResumeListQuery {
    type Error = String;

    fn try_from(params: ResumeListParams) -> Result<Self, Self::Error> {
        let limit = match single(&params.limit, "limit")? {
            None => RESUME_LIST_DEFAULT_LIMIT,
            Some(raw) => raw
                .parse::<i64>()
                .map_err(|_| "limit must be an integer".to_string())?,
        };
        if !(1..=RESUME_LIST_MAX_LIMIT).contains(&limit) {
            return Err(format!(
                "limit must be between 1 and {RESUME_LIST_MAX_LIMIT}"
            ));
        }

        let offset = match single(&params.offset, "offset")? {
            None => 0,
            Some(raw) => raw
                .parse::<i64>()
                .map_err(|_| "offset must be an integer".to_string())?,
        };
        if offset < 0 {
            return Err("offset must be >= 0".to_string());
        }

        let sort = match single(&params.sort, "sort")?.unwrap_or("id") {
            "id" => ResumeListSort::Id,
            "name" => ResumeListSort::Name,
            "updated_at" => ResumeListSort::UpdatedAt,
            other => return Err(format!("unknown sort field: {other}")),
        };

        let order = match single(&params.order, "order")?.unwrap_or("asc") {
            "asc" => ResumeListOrder::Asc,
            "desc" => ResumeListOrder::Desc,
            other => return Err(format!("unknown order: {other}")),
        };

        let search_terms = single(&params.search, "search")?
            .map(|raw| raw.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();

        Ok(ResumeListQuery {
            limit,
            offset,
            sort,
            order,
            search_terms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(
        limit: Option<&str>,
        offset: Option<&str>,
        sort: Option<&str>,
        order: Option<&str>,
        search: Option<&str>,
    ) -> ResumeListParams {
        fn opt(v: Option<&str>) -> Vec<String> {
            v.map(|s| vec![s.to_string()]).unwrap_or_default()
        }
        ResumeListParams {
            limit: opt(limit),
            offset: opt(offset),
            sort: opt(sort),
            order: opt(order),
            search: opt(search),
        }
    }

    #[test]
    fn defaults_apply_when_absent() {
        let query = ResumeListQuery::try_from(params(None, None, None, None, None)).unwrap();
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert_eq!(query.sort, ResumeListSort::Id);
        assert_eq!(query.order, ResumeListOrder::Asc);
        assert!(query.search_terms.is_empty());
    }

    #[test]
    fn rejects_out_of_range() {
        for bad in [
            params(Some("0"), None, None, None, None),
            params(Some("101"), None, None, None, None),
            params(Some("-3"), None, None, None, None),
            params(Some("abc"), None, None, None, None),
            params(None, Some("-1"), None, None, None),
            params(None, Some("xyz"), None, None, None),
            params(None, None, Some("bogus"), None, None),
            params(None, None, None, Some("sideways"), None),
        ] {
            assert!(ResumeListQuery::try_from(bad).is_err());
        }
    }

    #[test]
    fn search_splits_on_whitespace() {
        let query =
            ResumeListQuery::try_from(params(None, None, None, None, Some("  a  b c "))).unwrap();
        assert_eq!(query.search_terms, vec!["a", "b", "c"]);

        let blank = ResumeListQuery::try_from(params(None, None, None, None, Some("   "))).unwrap();
        assert!(blank.search_terms.is_empty());
    }
}
