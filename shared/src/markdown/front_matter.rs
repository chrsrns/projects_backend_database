use super::MarkdownError;

/// Keys allowed inside the optional `---` front-matter block. `resume_id` is
/// the import routing marker; the rest are variant targeting metadata.
const FRONT_MATTER_KEYS: &[&str] = &[
    "resume_id",
    "company_name",
    "role_title",
    "target_date",
    "job_description",
    "variant_label",
    "show_variant_tag",
];

/// Front-matter values decoded from the head block. `None` = key absent;
/// `Some(Value::Null)` covers both `key:` and `key: null`. Per-key type and
/// target rules are enforced by the import layer, not here.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrontMatter {
    pub resume_id: Option<serde_json::Value>,
    pub company_name: Option<serde_json::Value>,
    pub role_title: Option<serde_json::Value>,
    pub target_date: Option<serde_json::Value>,
    pub job_description: Option<serde_json::Value>,
    pub variant_label: Option<serde_json::Value>,
    pub show_variant_tag: Option<serde_json::Value>,
}

impl FrontMatter {
    /// True when any variant targeting metadata key is present. `resume_id`
    /// is a routing marker, not metadata.
    pub fn has_metadata(&self) -> bool {
        self.company_name.is_some()
            || self.role_title.is_some()
            || self.target_date.is_some()
            || self.job_description.is_some()
            || self.variant_label.is_some()
            || self.show_variant_tag.is_some()
    }
}

fn is_front_matter_fence(line: &str) -> bool {
    line.trim_end() == "---"
}

/// Detects and parses the optional front-matter block, returning the decoded
/// keys and the byte offset at which the markdown body starts. A file whose
/// first non-blank line is not a `---` fence is reported as body-only.
pub(crate) fn split_front_matter(
    markdown: &str,
) -> Result<(Option<FrontMatter>, usize), MarkdownError> {
    let after_bom = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let bom_len = markdown.len() - after_bom.len();

    let mut cursor = 0usize;
    loop {
        let rest = &after_bom[cursor..];
        let line_end = rest
            .find('\n')
            .map(|i| cursor + i)
            .unwrap_or(after_bom.len());
        let line = &after_bom[cursor..line_end];
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.trim().is_empty() {
            if line_end == after_bom.len() {
                return Ok((None, 0));
            }
            cursor = line_end + 1;
            continue;
        }
        if !is_front_matter_fence(line) {
            return Ok((None, 0));
        }
        cursor = line_end + 1;
        break;
    }

    let mut front_matter = FrontMatter::default();
    loop {
        if cursor >= after_bom.len() {
            return Err(MarkdownError::InvalidMarkdown(
                "Unclosed front-matter block: missing closing '---' line".to_string(),
            ));
        }
        let rest = &after_bom[cursor..];
        let line_end = rest
            .find('\n')
            .map(|i| cursor + i)
            .unwrap_or(after_bom.len());
        let line = &after_bom[cursor..line_end];
        let line = line.strip_suffix('\r').unwrap_or(line);
        let next = if line_end == after_bom.len() {
            after_bom.len()
        } else {
            line_end + 1
        };
        if is_front_matter_fence(line) {
            return Ok((Some(front_matter), bom_len + next));
        }
        if line.trim().is_empty() {
            return Err(MarkdownError::InvalidMarkdown(
                "Blank line inside front-matter block".to_string(),
            ));
        }
        let (key, raw_value) = line.split_once(':').ok_or_else(|| {
            MarkdownError::InvalidMarkdown(format!("Malformed front-matter line: '{}'", line))
        })?;
        let key = key.trim();
        let raw_value = raw_value.trim();
        if !FRONT_MATTER_KEYS.contains(&key) {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Unknown front-matter key '{}'. Expected one of: {}",
                key,
                FRONT_MATTER_KEYS.join(", ")
            )));
        }
        let value = if raw_value.is_empty() {
            serde_json::Value::Null
        } else {
            let value: serde_json::Value = serde_json::from_str(raw_value).map_err(|_| {
                MarkdownError::InvalidMarkdown(format!(
                    "Front-matter key '{}' value is not a JSON scalar: '{}'",
                    key, raw_value
                ))
            })?;
            if value.is_object() || value.is_array() {
                return Err(MarkdownError::InvalidMarkdown(format!(
                    "Front-matter key '{}' must be a JSON scalar, got '{}'",
                    key, raw_value
                )));
            }
            value
        };
        let slot = match key {
            "resume_id" => &mut front_matter.resume_id,
            "company_name" => &mut front_matter.company_name,
            "role_title" => &mut front_matter.role_title,
            "target_date" => &mut front_matter.target_date,
            "job_description" => &mut front_matter.job_description,
            "variant_label" => &mut front_matter.variant_label,
            "show_variant_tag" => &mut front_matter.show_variant_tag,
            _ => unreachable!("key checked against FRONT_MATTER_KEYS"),
        };
        if slot.is_some() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Duplicate front-matter key '{}'",
                key
            )));
        }
        *slot = Some(value);
        cursor = next;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parse_resume_markdown;

    #[test]
    fn test_front_matter_marker_parsed_and_stripped() {
        let markdown = "---\nresume_id: 42\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter present");
        assert_eq!(fm.resume_id, Some(serde_json::json!(42)));
        assert_eq!(parsed.resume.name, "Jane");
        assert_eq!(parsed.resume.email, "jane@example.com");
    }

    #[test]
    fn test_front_matter_detected_after_bom() {
        let markdown = "\u{feff}---\nresume_id: 7\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter present");
        assert_eq!(fm.resume_id, Some(serde_json::json!(7)));
        assert_eq!(parsed.resume.name, "Jane");
    }

    #[test]
    fn test_front_matter_detected_after_leading_blank_lines() {
        let markdown = "\n  \n---\nresume_id: 9\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter present");
        assert_eq!(fm.resume_id, Some(serde_json::json!(9)));
    }

    #[test]
    fn test_front_matter_empty_block_allowed() {
        let markdown = "---\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        assert!(parsed.front_matter.is_some());
        assert_eq!(parsed.resume.name, "Jane");
    }

    #[test]
    fn test_front_matter_fence_allows_trailing_whitespace() {
        let markdown = "---   \nresume_id: 5\n---\t\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        assert_eq!(
            parsed.front_matter.expect("front-matter").resume_id,
            Some(serde_json::json!(5))
        );
    }

    #[test]
    fn test_front_matter_metadata_keys_parsed() {
        let markdown = "---\nresume_id: 4\ncompany_name: \"Acme Corp\"\nrole_title: \"Engineer\"\ntarget_date: \"2026-03\"\njob_description: \"Own things\"\nvariant_label: \"v1\"\nshow_variant_tag: true\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter");
        assert_eq!(fm.company_name, Some(serde_json::json!("Acme Corp")));
        assert_eq!(fm.role_title, Some(serde_json::json!("Engineer")));
        assert_eq!(fm.target_date, Some(serde_json::json!("2026-03")));
        assert_eq!(fm.job_description, Some(serde_json::json!("Own things")));
        assert_eq!(fm.variant_label, Some(serde_json::json!("v1")));
        assert_eq!(fm.show_variant_tag, Some(serde_json::json!(true)));
    }

    #[test]
    fn test_front_matter_bare_key_is_null_value() {
        let markdown = "---\ncompany_name:\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter");
        assert_eq!(fm.company_name, Some(serde_json::Value::Null));
    }

    #[test]
    fn test_front_matter_unclosed_fence_is_error() {
        let markdown = "---\nresume_id: 4\n# Jane\n\n- Email: jane@example.com\n";
        assert!(parse_resume_markdown(markdown).is_err());
    }

    #[test]
    fn test_front_matter_duplicate_key_is_error() {
        let markdown =
            "---\nresume_id: 4\nresume_id: 5\n---\n# Jane\n\n- Email: jane@example.com\n";
        assert!(parse_resume_markdown(markdown).is_err());
    }

    #[test]
    fn test_front_matter_unknown_key_is_error() {
        let markdown = "---\nunknown_key: 4\n---\n# Jane\n\n- Email: jane@example.com\n";
        assert!(parse_resume_markdown(markdown).is_err());
    }

    #[test]
    fn test_front_matter_blank_line_inside_is_error() {
        let markdown =
            "---\nresume_id: 4\n\nresume_id: 5\n---\n# Jane\n\n- Email: jane@example.com\n";
        assert!(parse_resume_markdown(markdown).is_err());
    }

    #[test]
    fn test_front_matter_malformed_line_is_error() {
        let markdown = "---\nnot a key line\n---\n# Jane\n\n- Email: jane@example.com\n";
        assert!(parse_resume_markdown(markdown).is_err());
    }

    #[test]
    fn test_front_matter_non_scalar_value_is_error() {
        let markdown = "---\nresume_id: [1]\n---\n# Jane\n\n- Email: jane@example.com\n";
        assert!(parse_resume_markdown(markdown).is_err());
    }

    #[test]
    fn test_front_matter_key_whitespace_tolerated() {
        let markdown = "---\n  resume_id : 4 \n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        assert_eq!(
            parsed.front_matter.expect("front-matter").resume_id,
            Some(serde_json::json!(4))
        );
    }

    #[test]
    fn test_late_dash_fence_is_body_not_front_matter() {
        let markdown = "# Jane\n\n- Email: jane@example.com\n\n---\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        assert!(parsed.front_matter.is_none());
        assert_eq!(parsed.resume.name, "Jane");
    }

    #[test]
    fn test_front_matter_strips_before_section_parse() {
        let markdown = "---\nresume_id: 4\n---\n# Jane\n\n- Email: jane@example.com\n\n## Skills\n\n- Rust - 90%\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        assert_eq!(parsed.resume.skills.len(), 1);
        assert_eq!(parsed.resume.skills[0].skill_name, "Rust");
    }
}
