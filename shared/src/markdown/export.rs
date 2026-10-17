use super::dates::{format_markdown_date, format_optional_date};
use domain::models::{DatePrecision, FullResume, PartialDate};
use std::fmt::Write;
use std::str::FromStr;

fn json_scalar(value: &str) -> serde_json::Value {
    serde_json::Value::String(value.to_string())
}

fn write_front_matter_line(output: &mut String, key: &str, value: &serde_json::Value) {
    writeln!(output, "{}: {}", key, value).unwrap();
}

fn write_front_matter_text(output: &mut String, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        write_front_matter_line(output, key, &json_scalar(value));
    }
}

/// Emit suffix for inactive rows; active rows carry no marker.
fn active_suffix(active: bool) -> &'static str {
    if active { "" } else { " [active: false]" }
}

/// Serializes a resume to the markdown format. The head always carries a
/// `---` front-matter block with the `resume_id` marker; variant rows also
/// emit their stored non-null metadata keys, and `show_variant_tag` is emitted
/// only when the viewer owns the row.
pub fn resume_to_markdown(resume: &FullResume, viewer_is_owner: bool) -> String {
    let mut output = String::new();
    let row = &resume.resume;

    writeln!(output, "---").unwrap();
    write_front_matter_line(&mut output, "resume_id", &serde_json::json!(row.id));
    write_front_matter_text(&mut output, "company_name", &row.company_name);
    write_front_matter_text(&mut output, "role_title", &row.role_title);
    if let Some(target_date) = row.target_date {
        let partial = PartialDate {
            canonical: target_date,
            precision: row
                .target_date_precision
                .as_deref()
                .and_then(|value| DatePrecision::from_str(value).ok())
                .unwrap_or(DatePrecision::Day),
        };
        write_front_matter_line(
            &mut output,
            "target_date",
            &json_scalar(&partial.to_iso_string()),
        );
    }
    write_front_matter_text(&mut output, "job_description", &row.job_description);
    write_front_matter_text(&mut output, "variant_label", &row.variant_label);
    if row.base_resume_id.is_some() && viewer_is_owner {
        write_front_matter_line(
            &mut output,
            "show_variant_tag",
            &serde_json::Value::Bool(row.show_variant_tag),
        );
    }
    writeln!(output, "---").unwrap();
    writeln!(output).unwrap();

    writeln!(output, "# {}", resume.resume.name).unwrap();
    writeln!(output).unwrap();

    if let Some(location) = &resume.resume.location {
        writeln!(output, "- Location: {}", location).unwrap();
    }
    writeln!(output, "- Email: {}", resume.resume.email).unwrap();
    if let Some(profile_image_url) = &resume.resume.profile_image_url {
        writeln!(output, "- Profile Image: {}", profile_image_url).unwrap();
    }
    if let Some(github) = &resume.resume.github_url {
        writeln!(output, "- GitHub: {}", github).unwrap();
    }
    if let Some(mobile) = &resume.resume.mobile_number {
        writeln!(output, "- Mobile: {}", mobile).unwrap();
    }
    if let Some(video) = &resume.resume.video {
        writeln!(output, "- Video: {}", video).unwrap();
    }
    writeln!(output, "- Visibility: {}", resume.resume.visibility).unwrap();
    writeln!(output).unwrap();

    if let Some(summary) = &resume.resume.executive_summary {
        writeln!(output, "## Summary").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "{}", summary).unwrap();
        writeln!(output).unwrap();
    }

    if !resume.education.is_empty() {
        writeln!(output, "## Education").unwrap();
        writeln!(output).unwrap();
        for edu in &resume.education {
            writeln!(
                output,
                "### {} - {} ({} - {}) [order: {}]{}",
                edu.education_stage,
                edu.institution_name,
                format_markdown_date(
                    edu.start_date,
                    DatePrecision::from_str(&edu.start_date_precision)
                        .unwrap_or(DatePrecision::Day),
                ),
                format_optional_date(
                    edu.end_date,
                    edu.end_date_precision
                        .as_deref()
                        .and_then(|s| DatePrecision::from_str(s).ok()),
                ),
                edu.display_order.unwrap_or(0),
                active_suffix(edu.active)
            )
            .unwrap();

            if let Some(degree) = &edu.degree {
                writeln!(output, "- Degree: {}", degree).unwrap();
            }
            if let Some(description) = &edu.description {
                writeln!(output, "- Description: {}", description).unwrap();
            }

            if let Some(key_points) = resume.education_key_points.get(&edu.id) {
                for kp in key_points {
                    writeln!(output, "- {}{}", kp.key_point, active_suffix(kp.active)).unwrap();
                }
            }
            writeln!(output).unwrap();
        }
    }

    if !resume.skills.is_empty() {
        writeln!(output, "## Skills").unwrap();
        writeln!(output).unwrap();
        for skill in &resume.skills {
            writeln!(
                output,
                "- {} - {}% [order: {}]",
                skill.skill_name,
                skill.confidence_percentage,
                skill.display_order.unwrap_or(0)
            )
            .unwrap();
        }
        writeln!(output).unwrap();
    }

    if !resume.work_experiences.is_empty() {
        writeln!(output, "## Work Experience").unwrap();
        writeln!(output).unwrap();
        for work in &resume.work_experiences {
            write!(output, "### {}", work.job_title).unwrap();
            if let Some(company) = &work.company_name {
                write!(output, " - {}", company).unwrap();
            }
            writeln!(
                output,
                " ({} - {}) [order: {}]{}",
                format_markdown_date(
                    work.start_date,
                    DatePrecision::from_str(&work.start_date_precision)
                        .unwrap_or(DatePrecision::Day),
                ),
                format_optional_date(
                    work.end_date,
                    work.end_date_precision
                        .as_deref()
                        .and_then(|s| DatePrecision::from_str(s).ok()),
                ),
                work.display_order.unwrap_or(0),
                active_suffix(work.active)
            )
            .unwrap();

            if let Some(description) = &work.description {
                writeln!(output, "- Description: {}", description).unwrap();
            }

            if let Some(key_points) = resume.work_experience_key_points.get(&work.id) {
                for kp in key_points {
                    writeln!(output, "- {}{}", kp.key_point, active_suffix(kp.active)).unwrap();
                }
            }
            writeln!(output).unwrap();
        }
    }

    if !resume.portfolio_projects.is_empty() {
        writeln!(output, "## Portfolio Projects").unwrap();
        writeln!(output).unwrap();
        for project in &resume.portfolio_projects {
            writeln!(
                output,
                "### {} [order: {}]{}",
                project.project_name,
                project.display_order.unwrap_or(0),
                active_suffix(project.active)
            )
            .unwrap();

            if let Some(link) = &project.project_link {
                writeln!(output, "- Live: {}", link).unwrap();
            }
            if let Some(source) = &project.source_code_link {
                writeln!(output, "- Source: {}", source).unwrap();
            }
            if let Some(image_url) = &project.image_url {
                writeln!(output, "- Image: {}", image_url).unwrap();
            }
            if let Some(video_url) = &project.video_url {
                writeln!(output, "- Video: {}", video_url).unwrap();
            }

            if let Some(technologies) = resume.portfolio_technologies.get(&project.id)
                && !technologies.is_empty()
            {
                let names: Vec<String> = technologies
                    .iter()
                    .map(|t| format!("{}{}", t.technology_name, active_suffix(t.active)))
                    .collect();
                writeln!(output, "- Technologies: {}", names.join(", ")).unwrap();
            }

            if let Some(description) = &project.description {
                writeln!(output, "- Description: {}", description).unwrap();
            }

            if let Some(key_points) = resume.portfolio_key_points.get(&project.id) {
                for kp in key_points {
                    writeln!(output, "- {}{}", kp.key_point, active_suffix(kp.active)).unwrap();
                }
            }
            writeln!(output).unwrap();
        }
    }

    if !resume.languages.is_empty() {
        writeln!(output, "## Languages & Frameworks").unwrap();
        writeln!(output).unwrap();
        for language in &resume.languages {
            writeln!(
                output,
                "### {} [order: {}]",
                language.language_name,
                language.display_order.unwrap_or(0)
            )
            .unwrap();

            if let Some(frameworks) = resume.frameworks.get(&language.id) {
                for framework in frameworks {
                    writeln!(output, "- {}", framework.framework_name).unwrap();
                }
            }
            writeln!(output).unwrap();
        }
    }

    output
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::tests::{active_fixture, full_resume, resume_row};
    use chrono::NaiveDate;
    use domain::models::PartialDate;

    #[test]
    fn test_export_emits_resume_id_marker_for_base() {
        let output = resume_to_markdown(&full_resume(resume_row()), true);
        assert!(
            output.starts_with("---\nresume_id: \"00000000-0000-0000-0000-000000000007\"\n---\n")
        );
        assert!(output.contains("# Jane Doe"));
    }

    #[test]
    fn test_export_variant_emits_metadata_keys() {
        let mut resume = resume_row();
        resume.id = uuid::Uuid::from_u128(12);
        resume.base_resume_id = Some(uuid::Uuid::from_u128(7));
        resume.company_name = Some("Acme Corp".to_string());
        resume.role_title = Some("Engineer".to_string());
        resume.target_date = Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        resume.target_date_precision = Some("month".to_string());
        resume.job_description = Some("Line with \"quotes\"".to_string());
        resume.variant_label = Some("acme".to_string());
        resume.show_variant_tag = false;

        let output = resume_to_markdown(&full_resume(resume), true);
        assert!(output.contains("resume_id: \"00000000-0000-0000-0000-00000000000c\"\n"));
        assert!(output.contains("company_name: \"Acme Corp\"\n"));
        assert!(output.contains("role_title: \"Engineer\"\n"));
        assert!(output.contains("target_date: \"2026-03\"\n"));
        assert!(output.contains("job_description: \"Line with \\\"quotes\\\"\"\n"));
        assert!(output.contains("variant_label: \"acme\"\n"));
        assert!(output.contains("show_variant_tag: false\n"));
    }

    #[test]
    fn test_export_variant_omits_show_variant_tag_for_non_owner() {
        let mut resume = resume_row();
        resume.base_resume_id = Some(uuid::Uuid::from_u128(7));
        resume.company_name = Some("Acme".to_string());
        resume.show_variant_tag = false;

        let output = resume_to_markdown(&full_resume(resume), false);
        assert!(output.contains("company_name: \"Acme\"\n"));
        assert!(!output.contains("show_variant_tag"));
    }

    #[test]
    fn test_export_omits_null_metadata_keys() {
        let mut resume = resume_row();
        resume.base_resume_id = Some(uuid::Uuid::from_u128(7));
        resume.company_name = Some("Acme".to_string());

        let output = resume_to_markdown(&full_resume(resume), true);
        assert!(output.contains("company_name: \"Acme\"\n"));
        assert!(!output.contains("role_title:"));
        assert!(!output.contains("job_description:"));
    }

    #[test]
    fn test_export_emits_active_false_markers() {
        let output = resume_to_markdown(&active_fixture(false), true);
        assert!(output.contains("[order: 0] [active: false]"), "{}", output);
        assert!(output.contains("- Graduated [active: false]"), "{}", output);
        assert!(output.contains("- Did work [active: false]"), "{}", output);
        assert!(
            output.contains("- Built stuff [active: false]"),
            "{}",
            output
        );
        assert!(
            output.contains("- Technologies: Rust [active: false], Rocket"),
            "{}",
            output
        );
        assert!(!output.contains("[active: true]"), "{}", output);
    }

    #[test]
    fn test_export_emits_no_active_marker_on_active_rows() {
        let output = resume_to_markdown(&active_fixture(true), true);
        assert!(!output.contains("[active:"), "{}", output);
    }

    #[test]
    fn test_export_emits_visibility_bullet() {
        for value in ["private", "unlisted", "public"] {
            let mut row = resume_row();
            row.visibility = value.to_string();
            let output = resume_to_markdown(&full_resume(row), true);
            assert!(output.contains(&format!("- Visibility: {}", value)), "{}", output);
            assert!(!output.contains("- Public:"), "{}", output);
        }
    }

    #[test]
    fn test_export_visibility_round_trip() {
        for (value, expected) in [
            ("private", domain::models::Visibility::Private),
            ("unlisted", domain::models::Visibility::Unlisted),
            ("public", domain::models::Visibility::Public),
        ] {
            let mut row = resume_row();
            row.visibility = value.to_string();
            let output = resume_to_markdown(&full_resume(row), true);
            let parsed = crate::markdown::parse_resume_markdown(&output).expect("parse ok");
            assert_eq!(parsed.resume.visibility, expected, "{}", value);
        }
    }
}
