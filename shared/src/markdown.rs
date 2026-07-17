use chrono::NaiveDate;
use domain::models::{
    FullResume, ParsedEducation, ParsedEducationKeyPoint, ParsedFramework, ParsedLanguage,
    ParsedPortfolioKeyPoint, ParsedPortfolioProject, ParsedPortfolioTechnology, ParsedResume,
    ParsedSkill, ParsedWorkExperience, ParsedWorkExperienceKeyPoint,
};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownError {
    InvalidMarkdown(String),
}

const DATE_FORMAT: &str = "%b %Y";
const DATE_FORMAT_FALLBACK: &str = "%Y-%m-%d";

pub fn format_markdown_date(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}

pub fn parse_markdown_date(s: &str) -> Result<NaiveDate, MarkdownError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(MarkdownError::InvalidMarkdown(
            "Empty date string".to_string(),
        ));
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, DATE_FORMAT) {
        return Ok(date);
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, DATE_FORMAT_FALLBACK) {
        return Ok(date);
    }

    if let Some(date) = parse_month_year_date(trimmed) {
        return Ok(date);
    }

    Err(MarkdownError::InvalidMarkdown(format!(
        "Unable to parse date '{}'",
        trimmed
    )))
}

fn parse_month_year_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let month_num = match parts[0].to_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };

    let year: i32 = parts[1].parse().ok()?;
    NaiveDate::from_ymd_opt(year, month_num, 1)
}

fn format_optional_date(date: Option<NaiveDate>) -> String {
    match date {
        Some(d) => format_markdown_date(d),
        None => "Present".to_string(),
    }
}

fn try_parse_date_range(date_range: &str) -> Option<(NaiveDate, Option<NaiveDate>)> {
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
    if let Some(end) = end_date
        && start_date > end
    {
        return None;
    }
    Some((start_date, end_date))
}

fn find_last_date_range(
    text: &str,
) -> Result<(NaiveDate, Option<NaiveDate>, usize), MarkdownError> {
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

pub fn resume_to_markdown(resume: &FullResume) -> String {
    let mut output = String::new();

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
                format_markdown_date(edu.start_date),
                format_optional_date(edu.end_date),
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
                format_markdown_date(work.start_date),
                format_optional_date(work.end_date),
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
        assert_eq!(edu.start_date, NaiveDate::from_ymd_opt(2020, 9, 1).unwrap());
        assert_eq!(edu.end_date, NaiveDate::from_ymd_opt(2024, 5, 1));
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
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()
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
        assert_eq!(edu.start_date, NaiveDate::from_ymd_opt(2020, 9, 1).unwrap());
        assert_eq!(edu.end_date, NaiveDate::from_ymd_opt(2024, 5, 1));
    }

    #[test]
    fn test_work_experience_heading_with_iso_date() {
        let markdown = "# Resume\n\n- Email: test@example.com\n\n## Work Experience\n\n### Senior Engineer - Tech Corp (2020-01-15 - 2023-08-30)\n- Description: Backend development\n";
        let parsed = markdown_to_resume(markdown).expect("parse ok");
        let work = parsed.work_experiences.first().expect("one work");
        assert_eq!(work.start_date, NaiveDate::from_ymd_opt(2020, 1, 15).unwrap());
        assert_eq!(work.end_date, NaiveDate::from_ymd_opt(2023, 8, 30));
        assert_eq!(work.job_title, "Senior Engineer");
        assert_eq!(work.company_name.as_deref(), Some("Tech Corp"));
    }
}
