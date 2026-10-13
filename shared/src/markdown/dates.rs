use super::MarkdownError;
use chrono::NaiveDate;
use domain::models::{DatePrecision, PartialDate};

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

pub(crate) fn format_optional_date(
    date: Option<NaiveDate>,
    precision: Option<DatePrecision>,
) -> String {
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

pub(crate) fn find_last_date_range(
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
#[cfg(test)]
mod tests {
    use super::*;

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
}
