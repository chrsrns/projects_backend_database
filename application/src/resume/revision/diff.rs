use std::collections::BTreeMap;

use domain::models::{
    ResumeSnapshot, RevisionChange, RevisionChangeKind, SnapshotEducation,
    SnapshotEducationKeyPoint, SnapshotFramework, SnapshotLanguage, SnapshotPortfolioKeyPoint,
    SnapshotPortfolioProject, SnapshotPortfolioTechnology, SnapshotSkill, SnapshotWorkExperience,
    SnapshotWorkExperienceKeyPoint,
};
use serde::Serialize;
use serde_json::Value;

/// Collects the structural changes between two snapshots.
///
/// Paths are JSON Pointers into a positional view of the document: parent
/// collections are addressed by array index and child collections by position
/// within their parent, never by a stored child id. That is what keeps a diff
/// stable across a restore, which renumbers every child.
struct DiffBuilder {
    changes: Vec<RevisionChange>,
}

impl DiffBuilder {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn scalar<T: PartialEq + Serialize>(&mut self, path: &str, before: &T, after: &T) {
        if before == after {
            return;
        }

        self.changes.push(RevisionChange {
            path: path.to_string(),
            kind: RevisionChangeKind::Changed,
            before: Some(json(before)),
            after: Some(json(after)),
        });
    }

    fn added<T: Serialize>(&mut self, path: &str, after: &T) {
        self.changes.push(RevisionChange {
            path: path.to_string(),
            kind: RevisionChangeKind::Added,
            before: None,
            after: Some(json(after)),
        });
    }

    fn removed<T: Serialize>(&mut self, path: &str, before: &T) {
        self.changes.push(RevisionChange {
            path: path.to_string(),
            kind: RevisionChangeKind::Removed,
            before: Some(json(before)),
            after: None,
        });
    }
}

fn json<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

macro_rules! compare_fields {
    ($builder:expr, $path:expr, $before:expr, $after:expr, $($field:ident),+ $(,)?) => {
        $(
            $builder.scalar(
                &format!("{}/{}", $path, stringify!($field)),
                &$before.$field,
                &$after.$field,
            );
        )+
    };
}

/// Compares two positional collections: paired entries field by field, then
/// any tail as an addition or a removal of the whole entry.
fn diff_list<T: Serialize>(
    builder: &mut DiffBuilder,
    path: &str,
    before: &[T],
    after: &[T],
    compare: impl Fn(&mut DiffBuilder, &str, &T, &T),
) {
    let paired = before.len().min(after.len());

    for index in 0..paired {
        compare(
            builder,
            &format!("{}/{}", path, index),
            &before[index],
            &after[index],
        );
    }

    for (index, item) in before.iter().enumerate().skip(paired) {
        builder.removed(&format!("{}/{}", path, index), item);
    }

    for (index, item) in after.iter().enumerate().skip(paired) {
        builder.added(&format!("{}/{}", path, index), item);
    }
}

/// Compares a child collection whose entries are keyed by their parent's id.
///
/// Entries are matched by the parent's position, not by the key: after a
/// restore every child carries a fresh id, so keying on it would report the
/// whole collection as removed and re-added even when nothing changed.
/// One side of a child-collection comparison: the parent collection and the
/// children keyed by their parent's id.
struct Side<'a, P, C> {
    parents: &'a [P],
    children: &'a BTreeMap<i32, Vec<C>>,
}

fn diff_child_map<P, C: Serialize>(
    builder: &mut DiffBuilder,
    collection: &str,
    before: Side<'_, P, C>,
    after: Side<'_, P, C>,
    parent_id: impl Fn(&P) -> i32,
    compare: impl Fn(&mut DiffBuilder, &str, &C, &C),
) {
    let empty: Vec<C> = Vec::new();
    let positions = before.parents.len().max(after.parents.len());

    for position in 0..positions {
        let children_before = before
            .parents
            .get(position)
            .and_then(|parent| before.children.get(&parent_id(parent)))
            .unwrap_or(&empty);
        let children_after = after
            .parents
            .get(position)
            .and_then(|parent| after.children.get(&parent_id(parent)))
            .unwrap_or(&empty);

        let paired = children_before.len().min(children_after.len());

        for index in 0..paired {
            compare(
                builder,
                &format!("{}/{}/{}", collection, position, index),
                &children_before[index],
                &children_after[index],
            );
        }

        for (index, item) in children_before.iter().enumerate().skip(paired) {
            builder.removed(&format!("{}/{}/{}", collection, position, index), item);
        }

        for (index, item) in children_after.iter().enumerate().skip(paired) {
            builder.added(&format!("{}/{}/{}", collection, position, index), item);
        }
    }
}

fn compare_education(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotEducation,
    after: &SnapshotEducation,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        education_stage,
        institution_name,
        degree,
        start_date,
        end_date,
        description,
        display_order,
        active,
    );
}

fn compare_education_key_point(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotEducationKeyPoint,
    after: &SnapshotEducationKeyPoint,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        key_point,
        display_order,
        active,
    );
}

fn compare_skill(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotSkill,
    after: &SnapshotSkill,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        skill_name,
        confidence_percentage,
        display_order,
    );
}

fn compare_work_experience(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotWorkExperience,
    after: &SnapshotWorkExperience,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        job_title,
        company_name,
        start_date,
        end_date,
        description,
        display_order,
        active,
    );
}

fn compare_work_experience_key_point(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotWorkExperienceKeyPoint,
    after: &SnapshotWorkExperienceKeyPoint,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        key_point,
        display_order,
        active,
    );
}

fn compare_portfolio_project(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotPortfolioProject,
    after: &SnapshotPortfolioProject,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        project_name,
        image_url,
        project_link,
        source_code_link,
        video_url,
        description,
        display_order,
        active,
    );
}

fn compare_portfolio_key_point(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotPortfolioKeyPoint,
    after: &SnapshotPortfolioKeyPoint,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        key_point,
        display_order,
        active,
    );
}

fn compare_portfolio_technology(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotPortfolioTechnology,
    after: &SnapshotPortfolioTechnology,
) {
    compare_fields!(
        builder,
        path,
        before,
        after,
        technology_name,
        display_order,
        active,
    );
}

fn compare_language(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotLanguage,
    after: &SnapshotLanguage,
) {
    compare_fields!(builder, path, before, after, language_name, display_order,);
}

fn compare_framework(
    builder: &mut DiffBuilder,
    path: &str,
    before: &SnapshotFramework,
    after: &SnapshotFramework,
) {
    compare_fields!(builder, path, before, after, framework_name, display_order,);
}

/// The changes between the state a revision recorded and the state that
/// replaced it.
pub fn structural_diff(before: &ResumeSnapshot, after: &ResumeSnapshot) -> Vec<RevisionChange> {
    let mut builder = DiffBuilder::new();

    compare_fields!(
        builder,
        "",
        before,
        after,
        name,
        profile_image_url,
        location,
        email,
        github_url,
        mobile_number,
        executive_summary,
        video,
        visibility,
        company_name,
        role_title,
        target_date,
        target_date_precision,
        job_description,
        variant_label,
        show_variant_tag,
    );

    diff_list(
        &mut builder,
        "/education",
        &before.education,
        &after.education,
        compare_education,
    );
    diff_child_map(
        &mut builder,
        "/education_key_points",
        Side {
            parents: &before.education,
            children: &before.education_key_points,
        },
        Side {
            parents: &after.education,
            children: &after.education_key_points,
        },
        |parent| parent.id,
        compare_education_key_point,
    );
    diff_list(
        &mut builder,
        "/skills",
        &before.skills,
        &after.skills,
        compare_skill,
    );
    diff_list(
        &mut builder,
        "/work_experiences",
        &before.work_experiences,
        &after.work_experiences,
        compare_work_experience,
    );
    diff_child_map(
        &mut builder,
        "/work_experience_key_points",
        Side {
            parents: &before.work_experiences,
            children: &before.work_experience_key_points,
        },
        Side {
            parents: &after.work_experiences,
            children: &after.work_experience_key_points,
        },
        |parent| parent.id,
        compare_work_experience_key_point,
    );
    diff_list(
        &mut builder,
        "/portfolio_projects",
        &before.portfolio_projects,
        &after.portfolio_projects,
        compare_portfolio_project,
    );
    diff_child_map(
        &mut builder,
        "/portfolio_key_points",
        Side {
            parents: &before.portfolio_projects,
            children: &before.portfolio_key_points,
        },
        Side {
            parents: &after.portfolio_projects,
            children: &after.portfolio_key_points,
        },
        |parent| parent.id,
        compare_portfolio_key_point,
    );
    diff_child_map(
        &mut builder,
        "/portfolio_technologies",
        Side {
            parents: &before.portfolio_projects,
            children: &before.portfolio_technologies,
        },
        Side {
            parents: &after.portfolio_projects,
            children: &after.portfolio_technologies,
        },
        |parent| parent.id,
        compare_portfolio_technology,
    );
    diff_list(
        &mut builder,
        "/languages",
        &before.languages,
        &after.languages,
        compare_language,
    );
    diff_child_map(
        &mut builder,
        "/frameworks",
        Side {
            parents: &before.languages,
            children: &before.frameworks,
        },
        Side {
            parents: &after.languages,
            children: &after.frameworks,
        },
        |parent| parent.id,
        compare_framework,
    );

    builder.changes
}

/// A unified text diff over the canonical rendering of both documents, for
/// callers that want to show the change rather than interpret it.
pub fn unified_diff(before: &ResumeSnapshot, after: &ResumeSnapshot) -> String {
    let before_text = canonical_json(before);
    let after_text = canonical_json(after);

    similar::TextDiff::from_lines(&before_text, &after_text)
        .unified_diff()
        .header("before", "after")
        .to_string()
}

fn canonical_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::models::{DatePrecision, PartialDate, Visibility};

    fn snapshot() -> ResumeSnapshot {
        ResumeSnapshot {
            schema_version: 1,
            name: "Subject".to_string(),
            profile_image_url: None,
            location: Some("Here".to_string()),
            email: "subject@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: None,
            video: None,
            visibility: Visibility::Private,
            company_name: None,
            role_title: None,
            target_date: None,
            target_date_precision: None,
            job_description: None,
            variant_label: None,
            show_variant_tag: true,
            education: Vec::new(),
            education_key_points: BTreeMap::new(),
            skills: Vec::new(),
            work_experiences: Vec::new(),
            work_experience_key_points: BTreeMap::new(),
            portfolio_projects: Vec::new(),
            portfolio_key_points: BTreeMap::new(),
            portfolio_technologies: BTreeMap::new(),
            languages: Vec::new(),
            frameworks: BTreeMap::new(),
        }
    }

    fn education(id: i32, institution: &str) -> SnapshotEducation {
        SnapshotEducation {
            id,
            education_stage: "Bachelor".to_string(),
            institution_name: institution.to_string(),
            degree: None,
            start_date: PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 1, 1).expect("date"),
                precision: DatePrecision::Day,
            },
            end_date: None,
            description: None,
            display_order: Some(1),
            active: true,
        }
    }

    fn find<'a>(changes: &'a [RevisionChange], path: &str) -> Option<&'a RevisionChange> {
        changes.iter().find(|change| change.path == path)
    }

    #[test]
    fn json_pointer_paths_address_scalars_and_entries() {
        let before = snapshot();
        let mut after = snapshot();
        after.location = Some("There".to_string());
        after.education = vec![education(1, "University")];

        let changes = structural_diff(&before, &after);

        let location = find(&changes, "/location").expect("a change for /location");
        assert_eq!(location.kind, RevisionChangeKind::Changed);
        assert_eq!(
            location.before.as_ref().and_then(Value::as_str),
            Some("Here")
        );
        assert_eq!(
            location.after.as_ref().and_then(Value::as_str),
            Some("There")
        );

        let added = find(&changes, "/education/0").expect("an addition for the entry");
        assert_eq!(added.kind, RevisionChangeKind::Added);
        assert!(added.before.is_none());
        assert!(added.after.is_some());
    }

    #[test]
    fn positional_child_diff_ignores_stored_child_ids() {
        let mut before = snapshot();
        before.education = vec![education(41, "University")];
        let mut key_points = BTreeMap::new();
        key_points.insert(
            41,
            vec![SnapshotEducationKeyPoint {
                id: 900,
                key_point: "Graduated with honors".to_string(),
                display_order: Some(1),
                active: true,
            }],
        );
        before.education_key_points = key_points;

        let mut after = snapshot();
        after.education = vec![education(77, "University")];
        let mut key_points = BTreeMap::new();
        key_points.insert(
            77,
            vec![SnapshotEducationKeyPoint {
                id: 1200,
                key_point: "Graduated with honors".to_string(),
                display_order: Some(1),
                active: true,
            }],
        );
        after.education_key_points = key_points;

        let changes = structural_diff(&before, &after);

        assert!(
            changes.is_empty(),
            "renumbering every child with identical content is not a change: {:?}",
            changes
        );
    }

    #[test]
    fn unified_diff_renders_both_documents() {
        let before = snapshot();
        let mut after = snapshot();
        after.location = Some("There".to_string());

        let rendered = unified_diff(&before, &after);

        assert!(rendered.contains("--- before"));
        assert!(rendered.contains("+++ after"));
        assert!(
            rendered
                .lines()
                .any(|line| line.starts_with('-') && line.contains("Here")),
            "the removed line is rendered: {}",
            rendered
        );
        assert!(
            rendered
                .lines()
                .any(|line| line.starts_with('+') && line.contains("There")),
            "the added line is rendered: {}",
            rendered
        );
    }
}
