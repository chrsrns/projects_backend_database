use chrono::NaiveDate;
use domain::models::{
    DatePrecision, FullResume, ParsedEducation, ParsedEducationKeyPoint, ParsedFramework,
    ParsedLanguage, ParsedPortfolioKeyPoint, ParsedPortfolioProject, ParsedPortfolioTechnology,
    ParsedResume, ParsedSkill, ParsedWorkExperience, ParsedWorkExperienceKeyPoint, PartialDate,
};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::fmt::Write;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownError {
    InvalidMarkdown(String),
}

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

/// A markdown document split into its optional front-matter and the parsed
/// resume body.
#[derive(Debug)]
pub struct ParsedMarkdown {
    pub front_matter: Option<FrontMatter>,
    pub resume: ParsedResume,
}

fn is_front_matter_fence(line: &str) -> bool {
    line.trim_end() == "---"
}

/// Detects and parses the optional front-matter block, returning the decoded
/// keys and the byte offset at which the markdown body starts. A file whose
/// first non-blank line is not a `---` fence is reported as body-only.
fn split_front_matter(markdown: &str) -> Result<(Option<FrontMatter>, usize), MarkdownError> {
    let after_bom = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let bom_len = markdown.len() - after_bom.len();

    let mut cursor = 0usize;
    loop {
        let rest = &after_bom[cursor..];
        let line_end = rest.find('\n').map(|i| cursor + i).unwrap_or(after_bom.len());
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
        let line_end = rest.find('\n').map(|i| cursor + i).unwrap_or(after_bom.len());
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

/// Parses a resume markdown document, including its optional front-matter
/// block. The block is stripped before the body is parsed; a malformed block
/// is an error even when the body is well formed.
pub fn parse_resume_markdown(markdown: &str) -> Result<ParsedMarkdown, MarkdownError> {
    let (front_matter, body_start) = split_front_matter(markdown)?;
    let body = if front_matter.is_some() {
        &markdown[body_start..]
    } else {
        markdown
    };
    let resume = markdown_body_to_resume(body)?;
    Ok(ParsedMarkdown {
        front_matter,
        resume,
    })
}

fn json_scalar(value: &str) -> serde_json::Value {
    serde_json::Value::String(value.to_string())
}

pub fn format_markdown_date(date: NaiveDate, precision: DatePrecision) -> String {
    PartialDate {
        canonical: date,
        precision,
    }
    .to_markdown_string()
}

pub fn parse_markdown_date(s: &str) -> Result<PartialDate, MarkdownError> {
    PartialDate::from_markdown_str(s).map_err(MarkdownError::InvalidMarkdown)
}

fn format_optional_date(date: Option<NaiveDate>, precision: Option<DatePrecision>) -> String {
    match (date, precision) {
        (Some(d), Some(p)) => format_markdown_date(d, p),
        (Some(d), None) => format_markdown_date(d, DatePrecision::Day),
        (None, _) => "Present".to_string(),
    }
}

fn try_parse_date_range(date_range: &str) -> Option<(PartialDate, Option<PartialDate>)> {
    let (start_text, end_text) = date_range.split_once(" - ")?;
    let start_date = parse_markdown_date(start_text.trim()).ok()?;
    let trimmed_end = end_text.trim();
    let end_date = if trimmed_end.eq_ignore_ascii_case("present")
        || trimmed_end.eq_ignore_ascii_case("current")
        || trimmed_end.eq_ignore_ascii_case("ongoing")
    {
        None
    } else {
        Some(parse_markdown_date(trimmed_end).ok()?)
    };
    if let Some(ref end) = end_date
        && start_date.canonical_start_date() > end.canonical_end_date()
    {
        return None;
    }
    Some((start_date, end_date))
}

fn find_last_date_range(
    text: &str,
) -> Result<(PartialDate, Option<PartialDate>, usize), MarkdownError> {
    let mut candidates = Vec::new();
    for (close_pos, c) in text.char_indices() {
        if c == ')'
            && let Some(open_pos) = text[..close_pos].rfind('(')
        {
            candidates.push((open_pos, close_pos));
        }
    }

    for (open_pos, close_pos) in candidates.iter().rev() {
        let date_range = &text[open_pos + 1..*close_pos];
        if let Some((start_date, end_date)) = try_parse_date_range(date_range) {
            return Ok((start_date, end_date, *open_pos));
        }
    }

    Err(MarkdownError::InvalidMarkdown(format!(
        "Heading missing valid date range: '{}'",
        text
    )))
}

fn write_front_matter_line(output: &mut String, key: &str, value: &serde_json::Value) {
    writeln!(output, "{}: {}", key, value).unwrap();
}

fn write_front_matter_text(output: &mut String, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        write_front_matter_line(output, key, &json_scalar(value));
    }
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
    writeln!(output, "- Public: {}", resume.resume.is_public).unwrap();
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
                "### {} - {} ({} - {}) [order: {}]",
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
                edu.display_order.unwrap_or(0)
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
                    writeln!(output, "- {}", kp.key_point).unwrap();
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
                " ({} - {}) [order: {}]",
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
                work.display_order.unwrap_or(0)
            )
            .unwrap();

            if let Some(description) = &work.description {
                writeln!(output, "- Description: {}", description).unwrap();
            }

            if let Some(key_points) = resume.work_experience_key_points.get(&work.id) {
                for kp in key_points {
                    writeln!(output, "- {}", kp.key_point).unwrap();
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
                "### {} [order: {}]",
                project.project_name,
                project.display_order.unwrap_or(0)
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
                    .map(|t| t.technology_name.clone())
                    .collect();
                writeln!(output, "- Technologies: {}", names.join(", ")).unwrap();
            }

            if let Some(description) = &project.description {
                writeln!(output, "- Description: {}", description).unwrap();
            }

            if let Some(key_points) = resume.portfolio_key_points.get(&project.id) {
                for kp in key_points {
                    writeln!(output, "- {}", kp.key_point).unwrap();
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

pub fn markdown_to_resume(markdown: &str) -> Result<ParsedResume, MarkdownError> {
    Ok(parse_resume_markdown(markdown)?.resume)
}

fn markdown_body_to_resume(markdown: &str) -> Result<ParsedResume, MarkdownError> {
    let parser = Parser::new(markdown);
    let events: Vec<Event> = parser.collect();

    // Attach the resume info parsed from the top section.
    let mut resume_name: Option<String> = None;
    let mut location: Option<String> = None;
    let mut email: Option<String> = None;
    let mut profile_image_url: Option<String> = None;
    let mut github_url: Option<String> = None;
    let mut mobile_number: Option<String> = None;
    let mut is_public: bool = false;
    let mut executive_summary: Option<String> = None;
    let mut video: Option<String> = None;
    let mut summary_text = String::new();

    // Section buffers
    let mut education: Vec<ParsedEducation> = Vec::new();
    let mut education_key_points: Vec<ParsedEducationKeyPoint> = Vec::new();
    let mut skills: Vec<ParsedSkill> = Vec::new();
    let mut work_experiences: Vec<ParsedWorkExperience> = Vec::new();
    let mut work_experience_key_points: Vec<ParsedWorkExperienceKeyPoint> = Vec::new();
    let mut portfolio_projects: Vec<ParsedPortfolioProject> = Vec::new();
    let mut portfolio_key_points: Vec<ParsedPortfolioKeyPoint> = Vec::new();
    let mut portfolio_technologies: Vec<ParsedPortfolioTechnology> = Vec::new();
    let mut languages: Vec<ParsedLanguage> = Vec::new();
    let mut frameworks: Vec<ParsedFramework> = Vec::new();

    #[derive(Debug, PartialEq)]
    enum Section {
        Header,
        Summary,
        Education,
        Skills,
        WorkExperience,
        Portfolio,
        Languages,
    }

    let mut current_section = Section::Header;
    let mut current_education_index: Option<usize> = None;
    let mut current_work_index: Option<usize> = None;
    let mut current_portfolio_index: Option<usize> = None;
    let mut current_language_index: Option<usize> = None;

    let mut heading_level: HeadingLevel = HeadingLevel::H1;
    let mut heading_text = String::new();
    let mut list_item_text = String::new();
    let mut link_active = false;
    let mut item_had_link = false;
    let mut link_url = String::new();
    let mut link_text = String::new();

    for event in &events {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = *level;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if heading_level == HeadingLevel::H1 {
                    resume_name = Some(heading_text.trim().to_string());
                } else if heading_level == HeadingLevel::H2 {
                    if current_section == Section::Summary {
                        let s = summary_text.trim();
                        if !s.is_empty() {
                            executive_summary = Some(s.to_string());
                        }
                        summary_text.clear();
                    }
                    current_section = match heading_text.trim() {
                        "Education" => Section::Education,
                        "Skills" => Section::Skills,
                        "Work Experience" => Section::WorkExperience,
                        "Portfolio Projects" => Section::Portfolio,
                        "Languages & Frameworks" => Section::Languages,
                        "Summary" => Section::Summary,
                        other => {
                            return Err(MarkdownError::InvalidMarkdown(format!(
                                "Unknown section '{}'. Expected one of: Education, Skills, Work Experience, Portfolio Projects, Languages & Frameworks, Summary",
                                other
                            )));
                        }
                    };
                    current_education_index = None;
                    current_work_index = None;
                    current_portfolio_index = None;
                    current_language_index = None;
                } else if heading_level == HeadingLevel::H3 {
                    match current_section {
                        Section::Education => {
                            let parsed = parse_education_heading(&heading_text)?;
                            let idx = education.len();
                            current_education_index = Some(idx);
                            education.push(ParsedEducation {
                                education_stage: parsed.education_stage,
                                institution_name: parsed.institution_name,
                                degree: None,
                                start_date: parsed.start_date,
                                end_date: parsed.end_date,
                                description: None,
                                display_order: parsed.display_order,
                            });
                        }
                        Section::WorkExperience => {
                            let parsed = parse_work_experience_heading(&heading_text)?;
                            let idx = work_experiences.len();
                            current_work_index = Some(idx);
                            work_experiences.push(ParsedWorkExperience {
                                job_title: parsed.job_title,
                                company_name: parsed.company_name,
                                start_date: parsed.start_date,
                                end_date: parsed.end_date,
                                description: None,
                                display_order: parsed.display_order,
                            });
                        }
                        Section::Portfolio => {
                            let idx = portfolio_projects.len();
                            current_portfolio_index = Some(idx);
                            let display_order = extract_display_order(&heading_text);
                            portfolio_projects.push(ParsedPortfolioProject {
                                project_name: strip_order_suffix(&heading_text),
                                image_url: None,
                                project_link: None,
                                source_code_link: None,
                                video_url: None,
                                description: None,
                                display_order,
                            });
                        }
                        Section::Languages => {
                            let idx = languages.len();
                            current_language_index = Some(idx);
                            let display_order = extract_display_order(&heading_text);
                            languages.push(ParsedLanguage {
                                language_name: strip_order_suffix(&heading_text),
                                display_order,
                            });
                        }
                        _ => {}
                    }
                }
                heading_level = HeadingLevel::H1;
            }
            Event::Start(Tag::List(_)) => {}
            Event::Start(Tag::Item) => {
                list_item_text.clear();
                item_had_link = false;
            }
            Event::End(TagEnd::Item) => {
                let bullet = list_item_text.trim().to_string();
                if bullet.is_empty() {
                    continue;
                }

                match current_section {
                    Section::Header => {
                        if let Some((key, value)) = bullet.split_once(':') {
                            let key = key.trim();
                            let value = value.trim().to_string();
                            match key {
                                "Location" => location = Some(value),
                                "Email" => email = Some(value),
                                "Profile Image" => profile_image_url = Some(value),
                                "GitHub" => github_url = Some(value),
                                "Mobile" => mobile_number = Some(value),
                                "Video" => {
                                    if item_had_link {
                                        video = None;
                                    } else {
                                        video = Some(value);
                                    }
                                }
                                "Public" => {
                                    is_public = value.eq_ignore_ascii_case("true")
                                        || value.eq_ignore_ascii_case("yes")
                                        || value == "1";
                                }
                                _ => {}
                            }
                        }
                    }
                    Section::Skills => {
                        let skill = parse_skill_bullet(&bullet)?;
                        skills.push(ParsedSkill {
                            skill_name: skill.skill_name,
                            confidence_percentage: skill.confidence_percentage,
                            display_order: skill.display_order,
                        });
                    }
                    Section::Education => {
                        if let Some(edu) = education.last_mut() {
                            if let Some(degree) = bullet.strip_prefix("Degree:") {
                                edu.degree = Some(degree.trim().to_string());
                            } else if let Some(description) = bullet.strip_prefix("Description:") {
                                edu.description = Some(description.trim().to_string());
                            } else if let Some(idx) = current_education_index {
                                education_key_points.push(ParsedEducationKeyPoint {
                                    education_index: idx,
                                    key_point: bullet,
                                });
                            }
                        }
                    }
                    Section::WorkExperience => {
                        if let Some(work) = work_experiences.last_mut() {
                            if let Some(description) = bullet.strip_prefix("Description:") {
                                work.description = Some(description.trim().to_string());
                            } else if let Some(idx) = current_work_index {
                                work_experience_key_points.push(ParsedWorkExperienceKeyPoint {
                                    work_experience_index: idx,
                                    key_point: bullet,
                                });
                            }
                        }
                    }
                    Section::Portfolio => {
                        if let Some(project) = portfolio_projects.last_mut() {
                            if let Some(link_text) = bullet.strip_prefix("Live:") {
                                project.project_link = Some(link_text.trim().to_string());
                            } else if let Some(source_text) = bullet.strip_prefix("Source:") {
                                project.source_code_link = Some(source_text.trim().to_string());
                            } else if let Some(image_text) = bullet.strip_prefix("Image:") {
                                project.image_url = Some(image_text.trim().to_string());
                            } else if let Some(video_text) = bullet.strip_prefix("Video:") {
                                if item_had_link {
                                    project.video_url = None;
                                } else {
                                    project.video_url = Some(video_text.trim().to_string());
                                }
                            } else if let Some(tech_text) = bullet.strip_prefix("Technologies:") {
                                let tech_names: Vec<String> = tech_text
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                if let Some(idx) = current_portfolio_index {
                                    for name in tech_names {
                                        portfolio_technologies.push(ParsedPortfolioTechnology {
                                            portfolio_project_index: idx,
                                            technology_name: name,
                                        });
                                    }
                                }
                            } else if let Some(description) = bullet.strip_prefix("Description:") {
                                project.description = Some(description.trim().to_string());
                            } else if let Some(idx) = current_portfolio_index {
                                portfolio_key_points.push(ParsedPortfolioKeyPoint {
                                    portfolio_project_index: idx,
                                    key_point: bullet,
                                });
                            }
                        }
                    }
                    Section::Languages => {
                        if let Some(idx) = current_language_index {
                            frameworks.push(ParsedFramework {
                                language_index: idx,
                                framework_name: bullet,
                            });
                        }
                    }
                    Section::Summary => {}
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_active = true;
                item_had_link = true;
                link_url = dest_url.to_string();
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                if link_active {
                    let formatted = format!("{} ({})", link_text.trim(), link_url);
                    heading_text.push_str(&formatted);
                    list_item_text.push_str(&formatted);
                    if current_section == Section::Summary && heading_level == HeadingLevel::H1 {
                        summary_text.push_str(&formatted);
                    }
                    link_active = false;
                }
            }
            Event::Text(text) => {
                if link_active {
                    link_text.push_str(text);
                } else {
                    list_item_text.push_str(text);
                    heading_text.push_str(text);
                    if current_section == Section::Summary && heading_level == HeadingLevel::H1 {
                        summary_text.push_str(text);
                    }
                }
            }
            Event::Code(code) => {
                let formatted = format!("`{}`", code);
                if link_active {
                    link_text.push_str(&formatted);
                } else {
                    list_item_text.push_str(&formatted);
                    heading_text.push_str(&formatted);
                    if current_section == Section::Summary && heading_level == HeadingLevel::H1 {
                        summary_text.push_str(&formatted);
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if link_active {
                    link_text.push(' ');
                } else {
                    list_item_text.push(' ');
                    heading_text.push(' ');
                    if current_section == Section::Summary && heading_level == HeadingLevel::H1 {
                        summary_text.push(' ');
                    }
                }
            }
            _ => {}
        }
    }

    if current_section == Section::Summary {
        let s = summary_text.trim();
        if !s.is_empty() {
            executive_summary = Some(s.to_string());
        }
    }

    let resume_name = resume_name.ok_or_else(|| {
        MarkdownError::InvalidMarkdown("Missing resume name (H1 heading)".to_string())
    })?;

    let email = email.ok_or_else(|| {
        MarkdownError::InvalidMarkdown("Missing required field: Email".to_string())
    })?;
    validate_email(&email)?;

    let parsed = ParsedResume {
        name: resume_name,
        profile_image_url,
        location,
        email,
        github_url,
        mobile_number,
        executive_summary,
        video,
        is_public,
        education,
        education_key_points,
        skills,
        work_experiences,
        work_experience_key_points,
        portfolio_projects,
        portfolio_key_points,
        portfolio_technologies,
        languages,
        frameworks,
    };
    validate_child_indices(&parsed)?;
    Ok(parsed)
}

fn validate_child_indices(parsed: &ParsedResume) -> Result<(), MarkdownError> {
    for kp in &parsed.education_key_points {
        if kp.education_index >= parsed.education.len() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Education key point references missing education index {} (only {} available)",
                kp.education_index,
                parsed.education.len()
            )));
        }
    }
    for kp in &parsed.work_experience_key_points {
        if kp.work_experience_index >= parsed.work_experiences.len() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Work experience key point references missing work experience index {} (only {} available)",
                kp.work_experience_index,
                parsed.work_experiences.len()
            )));
        }
    }
    for kp in &parsed.portfolio_key_points {
        if kp.portfolio_project_index >= parsed.portfolio_projects.len() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Portfolio key point references missing project index {} (only {} available)",
                kp.portfolio_project_index,
                parsed.portfolio_projects.len()
            )));
        }
    }
    for tech in &parsed.portfolio_technologies {
        if tech.portfolio_project_index >= parsed.portfolio_projects.len() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Portfolio technology references missing project index {} (only {} available)",
                tech.portfolio_project_index,
                parsed.portfolio_projects.len()
            )));
        }
    }
    for fw in &parsed.frameworks {
        if fw.language_index >= parsed.languages.len() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Framework references missing language index {} (only {} available)",
                fw.language_index,
                parsed.languages.len()
            )));
        }
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), MarkdownError> {
    let parts: Vec<&str> = email.split('@').collect();
    let (local, domain) = match parts.as_slice() {
        [local, domain] => (*local, *domain),
        _ => {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Invalid email format: '{}'",
                email
            )));
        }
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err(MarkdownError::InvalidMarkdown(format!(
            "Invalid email format: '{}'",
            email
        )));
    }
    Ok(())
}

fn extract_display_order(text: &str) -> Option<i32> {
    let trimmed = text.trim_end();
    if let Some(start) = trimmed.rfind("[order:")
        && let Some(end) = trimmed[start..].find(']')
    {
        let after_bracket = &trimmed[start + end + 1..];
        if after_bracket.trim().is_empty() {
            let order_str = trimmed[start + 7..start + end].trim();
            return order_str.parse().ok();
        }
    }
    None
}

fn strip_order_suffix(text: &str) -> String {
    let trimmed = text.trim_end();
    if let Some(start) = trimmed.rfind("[order:")
        && let Some(end) = trimmed[start..].find(']')
    {
        let after_bracket = &trimmed[start + end + 1..];
        if after_bracket.trim().is_empty() {
            return trimmed[..start].trim_end().to_string();
        }
    }
    text.to_string()
}

fn parse_education_heading(text: &str) -> Result<ParsedEducation, MarkdownError> {
    let text = text.trim();
    let (start_date, end_date, paren_open) = find_last_date_range(text)?;
    let header_part = text[..paren_open].trim();

    let separator = " - ";
    let sep_pos = header_part.find(separator).ok_or_else(|| {
        MarkdownError::InvalidMarkdown(format!(
            "Education heading missing separator '{}': '{}'",
            separator, header_part
        ))
    })?;
    let education_stage = header_part[..sep_pos].trim().to_string();
    let institution_name = header_part[sep_pos + separator.len()..].trim().to_string();

    if education_stage.is_empty() || institution_name.is_empty() {
        return Err(MarkdownError::InvalidMarkdown(format!(
            "Education heading has empty stage or institution: '{}'",
            text
        )));
    }

    let display_order = extract_display_order(text);

    Ok(ParsedEducation {
        education_stage,
        institution_name,
        degree: None,
        start_date,
        end_date,
        description: None,
        display_order,
    })
}

fn parse_work_experience_heading(text: &str) -> Result<ParsedWorkExperience, MarkdownError> {
    let text = text.trim();
    let (start_date, end_date, paren_open) = find_last_date_range(text)?;
    let title_part = text[..paren_open].trim();

    let separator = " - ";
    let (job_title, company_name) = if let Some(sep_pos) = title_part.find(separator) {
        (
            title_part[..sep_pos].trim().to_string(),
            Some(title_part[sep_pos + separator.len()..].trim().to_string()),
        )
    } else {
        (title_part.to_string(), None)
    };

    if job_title.is_empty() {
        return Err(MarkdownError::InvalidMarkdown(format!(
            "Work experience heading has empty job title: '{}'",
            text
        )));
    }

    let display_order = extract_display_order(text);

    Ok(ParsedWorkExperience {
        job_title,
        company_name,
        start_date,
        end_date,
        description: None,
        display_order,
    })
}

fn parse_skill_bullet(text: &str) -> Result<ParsedSkill, MarkdownError> {
    let text = text.trim();
    if let Some(pos) = text.rfind(" - ") {
        let name = text[..pos].trim().to_string();
        let pct_part = strip_order_suffix(text[pos + 3..].trim());
        let pct_str = pct_part.trim_end_matches('%').trim();
        let confidence_percentage = pct_str.parse().map_err(|_| {
            MarkdownError::InvalidMarkdown(format!(
                "Skill bullet has invalid percentage: '{}'",
                pct_str
            ))
        })?;
        if name.is_empty() {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Skill bullet has empty skill name: '{}'",
                text
            )));
        }
        if !(0..=100).contains(&confidence_percentage) {
            return Err(MarkdownError::InvalidMarkdown(format!(
                "Skill bullet has confidence percentage out of range (0-100): '{}'",
                confidence_percentage
            )));
        }
        let display_order = extract_display_order(text);
        return Ok(ParsedSkill {
            skill_name: name,
            confidence_percentage,
            display_order,
        });
    }
    Err(MarkdownError::InvalidMarkdown(format!(
        "Skill bullet missing ' - ' separator: '{}'",
        text
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::models::{DatePrecision, PartialDate};

    #[test]
    fn test_heading_preserves_inline_code() {
        let markdown = "# Jane `Code` Doe\n\n- Email: test@example.com\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        assert_eq!(parsed.name, "Jane `Code` Doe");
    }

    #[test]
    fn test_heading_preserves_link() {
        let markdown = "# [Jane Doe](https://example.com)\n\n- Email: test@example.com\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        assert_eq!(parsed.name, "Jane Doe (https://example.com)");
    }

    #[test]
    fn test_work_experience_heading_preserves_inline_code_and_link() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Work Experience\n\n### [Senior `Rust` Engineer](https://example.com/job) - Tech Corp (Jan 2020 - Present)\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let work = parsed
            .work_experiences
            .first()
            .expect("one work experience");
        assert_eq!(
            work.job_title,
            "Senior `Rust` Engineer (https://example.com/job)"
        );
        assert_eq!(work.company_name.as_deref(), Some("Tech Corp"));
    }

    #[test]
    fn test_skill_name_with_brackets_is_preserved() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Skills\n\n- Rust [web] - 90%\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let skill = parsed.skills.first().expect("one skill");
        assert_eq!(skill.skill_name, "Rust [web]");
        assert_eq!(skill.confidence_percentage, 90);
    }

    #[test]
    fn test_order_suffix_extraction_is_consistent_with_multiple_markers() {
        let text = "Rust [order: 1] extra [order: 2]";
        assert_eq!(extract_display_order(text), Some(2));
        assert_eq!(strip_order_suffix(text), "Rust [order: 1] extra");
    }

    #[test]
    fn test_non_trailing_order_marker_is_not_extracted() {
        let text = "Rust [order: not a number] extra";
        assert_eq!(extract_display_order(text), None);
        assert_eq!(strip_order_suffix(text), "Rust [order: not a number] extra");
    }

    #[test]
    fn test_order_marker_without_closing_bracket_is_not_extracted() {
        let text = "Rust [order: 1";
        assert_eq!(extract_display_order(text), None);
        assert_eq!(strip_order_suffix(text), "Rust [order: 1");
    }

    #[test]
    fn test_bracketed_order_text_in_skill_name_is_preserved() {
        let markdown =
            "# Resume\n\n- Email: test@example.com\n\n## Skills\n\n- Rust [order: custom] - 90%\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let skill = parsed.skills.first().expect("one skill");
        assert_eq!(skill.skill_name, "Rust [order: custom]");
        assert_eq!(skill.confidence_percentage, 90);
    }

    #[test]
    fn test_orphaned_education_key_point_returns_error() {
        let parsed = ParsedResume {
            name: "Test".to_string(),
            profile_image_url: None,
            location: None,
            email: "test@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: None,
            video: None,
            is_public: false,
            education: vec![],
            education_key_points: vec![ParsedEducationKeyPoint {
                education_index: 0,
                key_point: "Graduated".to_string(),
            }],
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: vec![],
            portfolio_projects: vec![],
            portfolio_key_points: vec![],
            portfolio_technologies: vec![],
            languages: vec![],
            frameworks: vec![],
        };
        assert!(validate_child_indices(&parsed).is_err());
    }

    #[test]
    fn test_orphaned_work_key_point_returns_error() {
        let parsed = ParsedResume {
            name: "Test".to_string(),
            profile_image_url: None,
            location: None,
            email: "test@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: None,
            video: None,
            is_public: false,
            education: vec![],
            education_key_points: vec![],
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: vec![ParsedWorkExperienceKeyPoint {
                work_experience_index: 0,
                key_point: "Did work".to_string(),
            }],
            portfolio_projects: vec![],
            portfolio_key_points: vec![],
            portfolio_technologies: vec![],
            languages: vec![],
            frameworks: vec![],
        };
        assert!(validate_child_indices(&parsed).is_err());
    }

    #[test]
    fn test_orphaned_portfolio_key_point_returns_error() {
        let parsed = ParsedResume {
            name: "Test".to_string(),
            profile_image_url: None,
            location: None,
            email: "test@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: None,
            video: None,
            is_public: false,
            education: vec![],
            education_key_points: vec![],
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: vec![],
            portfolio_projects: vec![],
            portfolio_key_points: vec![ParsedPortfolioKeyPoint {
                portfolio_project_index: 0,
                key_point: "Built stuff".to_string(),
            }],
            portfolio_technologies: vec![],
            languages: vec![],
            frameworks: vec![],
        };
        assert!(validate_child_indices(&parsed).is_err());
    }

    #[test]
    fn test_orphaned_framework_returns_error() {
        let parsed = ParsedResume {
            name: "Test".to_string(),
            profile_image_url: None,
            location: None,
            email: "test@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            executive_summary: None,
            video: None,
            is_public: false,
            education: vec![],
            education_key_points: vec![],
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: vec![],
            portfolio_projects: vec![],
            portfolio_key_points: vec![],
            portfolio_technologies: vec![],
            languages: vec![],
            frameworks: vec![ParsedFramework {
                language_index: 0,
                framework_name: "Rocket".to_string(),
            }],
        };
        assert!(validate_child_indices(&parsed).is_err());
    }

    #[test]
    fn test_education_heading_with_parentheses_in_institution() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Education\n\n### Bachelor's - University of (ABC) (Sep 2020 - May 2024)\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let edu = parsed.education.first().expect("one education");
        assert_eq!(edu.education_stage, "Bachelor's");
        assert_eq!(edu.institution_name, "University of (ABC)");
        assert_eq!(
            edu.start_date,
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                precision: DatePrecision::Month,
            }
        );
        assert_eq!(
            edu.end_date,
            Some(PartialDate {
                canonical: NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
                precision: DatePrecision::Month,
            })
        );
    }

    #[test]
    fn test_work_experience_heading_with_parentheses_in_company() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Work Experience\n\n### Senior Engineer - Acme (Global) (Jan 2020 - Present)\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let work = parsed
            .work_experiences
            .first()
            .expect("one work experience");
        assert_eq!(work.job_title, "Senior Engineer");
        assert_eq!(work.company_name.as_deref(), Some("Acme (Global)"));
        assert_eq!(
            work.start_date,
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                precision: DatePrecision::Month,
            }
        );
        assert_eq!(work.end_date, None);
    }

    #[test]
    fn test_education_heading_without_valid_date_range() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Education\n\n### Bachelor's - University (ABC)\n";
        assert!(markdown_to_resume(markdown).is_err());
    }

    #[test]
    fn test_work_experience_heading_without_valid_date_range() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Work Experience\n\n### Senior Engineer - Company (ABC)\n";
        assert!(markdown_to_resume(markdown).is_err());
    }

    #[test]
    fn test_education_heading_with_iso_date() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Education\n\n### Bachelor's - University of ABC (2020-09-01 - 2024-05-01)\n- Degree: Bachelor of Science\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let edu = parsed.education.first().expect("one education");
        assert_eq!(
            edu.start_date,
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                precision: DatePrecision::Day,
            }
        );
        assert_eq!(
            edu.end_date,
            Some(PartialDate {
                canonical: NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
                precision: DatePrecision::Day,
            })
        );
    }

    #[test]
    fn test_work_experience_heading_with_iso_date() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Work Experience\n\n### Senior Engineer - Tech Corp (2020-01-15 - 2023-08-30)\n- Description: Backend development\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let work = parsed.work_experiences.first().expect("one work");
        assert_eq!(
            work.start_date,
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                precision: DatePrecision::Day,
            }
        );
        assert_eq!(
            work.end_date,
            Some(PartialDate {
                canonical: NaiveDate::from_ymd_opt(2023, 8, 30).unwrap(),
                precision: DatePrecision::Day,
            })
        );
        assert_eq!(work.job_title, "Senior Engineer");
        assert_eq!(work.company_name.as_deref(), Some("Tech Corp"));
    }

    #[test]
    fn test_education_heading_with_full_month_name() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Education\n\n### Bachelor's in Computer Science - University of ABC (September 2020 - May 2024)\n- Degree: Bachelor of Science\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let edu = parsed.education.first().expect("one education");
        assert_eq!(
            edu.start_date,
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                precision: DatePrecision::Month,
            }
        );
        assert_eq!(
            edu.end_date,
            Some(PartialDate {
                canonical: NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
                precision: DatePrecision::Month,
            })
        );
    }

    #[test]
    fn test_work_experience_heading_with_full_month_name() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Work Experience\n\n### Senior Engineer - Tech Corp (January 1919 - Present)\n- Description: Backend development\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let work = parsed.work_experiences.first().expect("one work");
        assert_eq!(
            work.start_date,
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(1919, 1, 1).unwrap(),
                precision: DatePrecision::Month,
            }
        );
        assert_eq!(work.end_date, None);
        assert_eq!(work.job_title, "Senior Engineer");
        assert_eq!(work.company_name.as_deref(), Some("Tech Corp"));
    }

    #[test]
    fn test_parse_markdown_date_accepts_full_month_names() {
        assert_eq!(
            parse_markdown_date("January 1919").unwrap(),
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(1919, 1, 1).unwrap(),
                precision: DatePrecision::Month,
            }
        );
        assert_eq!(
            parse_markdown_date("september 2020").unwrap(),
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                precision: DatePrecision::Month,
            }
        );
    }

    #[test]
    fn test_parse_markdown_date_year_only() {
        assert_eq!(
            parse_markdown_date("2020").unwrap(),
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                precision: DatePrecision::Year,
            }
        );
    }

    #[test]
    fn test_parse_markdown_date_full_date() {
        assert_eq!(
            parse_markdown_date("2020-09-01").unwrap(),
            PartialDate {
                canonical: NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                precision: DatePrecision::Day,
            }
        );
    }

    #[test]
    fn test_parse_markdown_date_rejects_iso_month() {
        assert!(parse_markdown_date("2020-09").is_err());
    }

    #[test]
    fn test_invalid_date_range_is_rejected() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Education\n\n### Bachelor's - University of ABC (Sep 2024 - May 2020)\n";
        assert!(markdown_to_resume(markdown).is_err());
    }

    #[test]
    fn test_format_markdown_date_outputs_by_precision() {
        assert_eq!(
            format_markdown_date(
                NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                DatePrecision::Year
            ),
            "2020"
        );
        assert_eq!(
            format_markdown_date(
                NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                DatePrecision::Month
            ),
            "Sep 2020"
        );
        assert_eq!(
            format_markdown_date(
                NaiveDate::from_ymd_opt(2020, 9, 1).unwrap(),
                DatePrecision::Day
            ),
            "2020-09-01"
        );
    }

    #[test]
    fn test_format_optional_date_outputs_present_or_formatted() {
        assert_eq!(
            format_optional_date(
                Some(NaiveDate::from_ymd_opt(2020, 9, 1).unwrap()),
                Some(DatePrecision::Month)
            ),
            "Sep 2020"
        );
        assert_eq!(format_optional_date(None, None), "Present");
    }

    #[test]
    fn test_parsed_portfolio_project_has_video_url() {
        let project = ParsedPortfolioProject {
            project_name: "Demo".to_string(),
            image_url: None,
            project_link: None,
            source_code_link: None,
            video_url: Some("https://example.com/video.mp4".to_string()),
            description: None,
            display_order: None,
        };
        assert_eq!(
            project.video_url.as_deref(),
            Some("https://example.com/video.mp4")
        );
    }

    #[test]
    fn test_markdown_parses_portfolio_video_url() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Portfolio Projects\n\n### My Portfolio\n- Image: https://example.com/image.png\n- Video: https://example.com/video.mp4\n- Technologies: Rust\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let project = parsed.portfolio_projects.first().expect("one project");
        assert_eq!(
            project.video_url.as_deref(),
            Some("https://example.com/video.mp4")
        );
    }

    #[test]
    fn test_markdown_ignores_portfolio_video_url_markdown_link() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Portfolio Projects\n\n### My Portfolio\n- Video: [Watch](https://example.com/video.mp4)\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let project = parsed.portfolio_projects.first().expect("one project");
        assert_eq!(project.video_url, None);
    }

    fn resume_row() -> domain::models::Resume {
        domain::models::Resume {
            id: 7,
            name: "Jane Doe".to_string(),
            profile_image_url: None,
            location: None,
            email: "jane@example.com".to_string(),
            github_url: None,
            mobile_number: None,
            created_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_by: Some(1),
            is_public: true,
            executive_summary: None,
            video: None,
            base_resume_id: None,
            company_name: None,
            role_title: None,
            target_date: None,
            target_date_precision: None,
            job_description: None,
            variant_label: None,
            show_variant_tag: true,
        }
    }

    fn full_resume(resume: domain::models::Resume) -> FullResume {
        FullResume {
            resume,
            education: vec![],
            education_key_points: std::collections::HashMap::new(),
            skills: vec![],
            work_experiences: vec![],
            work_experience_key_points: std::collections::HashMap::new(),
            portfolio_projects: vec![],
            portfolio_key_points: std::collections::HashMap::new(),
            portfolio_technologies: std::collections::HashMap::new(),
            languages: vec![],
            frameworks: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_front_matter_marker_parsed_and_stripped() {
        let markdown =
            "---\nresume_id: 42\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter present");
        assert_eq!(fm.resume_id, Some(serde_json::json!(42)));
        assert_eq!(parsed.resume.name, "Jane");
        assert_eq!(parsed.resume.email, "jane@example.com");
    }

    #[test]
    fn test_front_matter_detected_after_bom() {
        let markdown =
            "\u{feff}---\nresume_id: 7\n---\n# Jane\n\n- Email: jane@example.com\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        let fm = parsed.front_matter.expect("front-matter present");
        assert_eq!(fm.resume_id, Some(serde_json::json!(7)));
        assert_eq!(parsed.resume.name, "Jane");
    }

    #[test]
    fn test_front_matter_detected_after_leading_blank_lines() {
        let markdown =
            "\n  \n---\nresume_id: 9\n---\n# Jane\n\n- Email: jane@example.com\n";
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
        let markdown =
            "---   \nresume_id: 5\n---\t\n# Jane\n\n- Email: jane@example.com\n";
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
        let markdown =
            "---\nunknown_key: 4\n---\n# Jane\n\n- Email: jane@example.com\n";
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
        let markdown =
            "---\n  resume_id : 4 \n---\n# Jane\n\n- Email: jane@example.com\n";
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
        let markdown =
            "---\nresume_id: 4\n---\n# Jane\n\n- Email: jane@example.com\n\n## Skills\n\n- Rust - 90%\n";
        let parsed = parse_resume_markdown(markdown).expect("parse ok");
        assert_eq!(parsed.resume.skills.len(), 1);
        assert_eq!(parsed.resume.skills[0].skill_name, "Rust");
    }

    #[test]
    fn test_export_emits_resume_id_marker_for_base() {
        let output = resume_to_markdown(&full_resume(resume_row()), true);
        assert!(output.starts_with("---\nresume_id: 7\n---\n"));
        assert!(output.contains("# Jane Doe"));
    }

    #[test]
    fn test_export_variant_emits_metadata_keys() {
        let mut resume = resume_row();
        resume.id = 12;
        resume.base_resume_id = Some(7);
        resume.company_name = Some("Acme Corp".to_string());
        resume.role_title = Some("Engineer".to_string());
        resume.target_date = Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        resume.target_date_precision = Some("month".to_string());
        resume.job_description = Some("Line with \"quotes\"".to_string());
        resume.variant_label = Some("acme".to_string());
        resume.show_variant_tag = false;

        let output = resume_to_markdown(&full_resume(resume), true);
        assert!(output.contains("resume_id: 12\n"));
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
        resume.base_resume_id = Some(7);
        resume.company_name = Some("Acme".to_string());
        resume.show_variant_tag = false;

        let output = resume_to_markdown(&full_resume(resume), false);
        assert!(output.contains("company_name: \"Acme\"\n"));
        assert!(!output.contains("show_variant_tag"));
    }

    #[test]
    fn test_export_omits_null_metadata_keys() {
        let mut resume = resume_row();
        resume.base_resume_id = Some(7);
        resume.company_name = Some("Acme".to_string());

        let output = resume_to_markdown(&full_resume(resume), true);
        assert!(output.contains("company_name: \"Acme\"\n"));
        assert!(!output.contains("role_title:"));
        assert!(!output.contains("job_description:"));
    }

    #[test]
    fn test_export_import_round_trip_front_matter() {
        let mut resume = resume_row();
        resume.id = 21;
        resume.base_resume_id = Some(7);
        resume.company_name = Some("Acme".to_string());

        let exported = resume_to_markdown(&full_resume(resume), true);
        let parsed = parse_resume_markdown(&exported).expect("round-trip parse ok");
        let fm = parsed.front_matter.expect("front-matter");
        assert_eq!(fm.resume_id, Some(serde_json::json!(21)));
        assert_eq!(fm.company_name, Some(serde_json::json!("Acme")));
        assert_eq!(parsed.resume.name, "Jane Doe");
    }
}
