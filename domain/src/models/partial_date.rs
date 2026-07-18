use chrono::{Datelike, NaiveDate};
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::de::{self, Visitor};
use std::fmt;
use std::str::FromStr;
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{ObjectBuilder, SchemaType, Type};
use utoipa::{PartialSchema, ToSchema};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

impl fmt::Display for DatePrecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DatePrecision::Year => "year",
            DatePrecision::Month => "month",
            DatePrecision::Day => "day",
        })
    }
}

impl FromStr for DatePrecision {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "year" => Ok(DatePrecision::Year),
            "month" => Ok(DatePrecision::Month),
            "day" => Ok(DatePrecision::Day),
            _ => Err(format!("invalid date precision '{}'", s)),
        }
    }
}

impl PartialSchema for DatePrecision {
    fn schema() -> RefOr<utoipa::openapi::schema::Schema> {
        ObjectBuilder::new()
            .schema_type(SchemaType::new(Type::String))
            .enum_values(Some(["day", "month", "year"]))
            .description(Some("Date precision"))
            .into()
    }
}

impl ToSchema for DatePrecision {
    fn name() -> std::borrow::Cow<'static, str> {
        "DatePrecision".into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartialDate {
    pub canonical: NaiveDate,
    pub precision: DatePrecision,
}

impl PartialDate {
    pub fn from_iso_str(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("empty date string".to_string());
        }

        if let Some(date) = parse_year(trimmed) {
            return date;
        }

        if let Some(date) = parse_month_iso(trimmed) {
            return date;
        }

        if let Some(date) = parse_day(trimmed) {
            return date;
        }

        Err(format!("invalid partial date '{}'", trimmed))
    }

    pub fn from_markdown_str(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("empty date string".to_string());
        }

        if let Some(date) = parse_year(trimmed) {
            return date;
        }

        if let Some(date) = parse_day(trimmed) {
            return date;
        }

        if let Some(date) = parse_month_name(trimmed) {
            return date;
        }

        Err(format!("unable to parse date '{}'", trimmed))
    }

    pub fn canonical_start_date(&self) -> NaiveDate {
        self.canonical
    }

    pub fn canonical_end_date(&self) -> NaiveDate {
        match self.precision {
            DatePrecision::Year => {
                NaiveDate::from_ymd_opt(self.canonical.year(), 12, 31).expect("valid year end")
            }
            DatePrecision::Month => {
                last_day_of_month(self.canonical.year(), self.canonical.month())
            }
            DatePrecision::Day => self.canonical,
        }
    }

    pub fn to_iso_string(&self) -> String {
        match self.precision {
            DatePrecision::Year => self.canonical.format("%Y").to_string(),
            DatePrecision::Month => self.canonical.format("%Y-%m").to_string(),
            DatePrecision::Day => self.canonical.format("%Y-%m-%d").to_string(),
        }
    }

    pub fn to_markdown_string(&self) -> String {
        match self.precision {
            DatePrecision::Year => self.canonical.format("%Y").to_string(),
            DatePrecision::Month => self.canonical.format("%b %Y").to_string(),
            DatePrecision::Day => self.canonical.format("%Y-%m-%d").to_string(),
        }
    }
}

impl FromStr for PartialDate {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_iso_str(s)
    }
}

impl Serialize for PartialDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_iso_string())
    }
}

impl<'de> Deserialize<'de> for PartialDate {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PartialDateVisitor;

        impl<'de> Visitor<'de> for PartialDateVisitor {
            type Value = PartialDate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an ISO partial date string (YYYY, YYYY-MM, or YYYY-MM-DD)")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                value
                    .parse::<PartialDate>()
                    .map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(PartialDateVisitor)
    }
}

impl PartialSchema for PartialDate {
    fn schema() -> RefOr<utoipa::openapi::schema::Schema> {
        ObjectBuilder::new()
            .schema_type(SchemaType::new(Type::String))
            .pattern(Some(r"^\d{4}(-\d{2}(-\d{2})?)?$"))
            .description(Some("ISO partial date: YYYY, YYYY-MM, or YYYY-MM-DD"))
            .into()
    }
}

impl ToSchema for PartialDate {
    fn name() -> std::borrow::Cow<'static, str> {
        "PartialDate".into()
    }
}

pub fn deserialize_optional_nullable_partial_date<'de, D>(
    deserializer: D,
) -> Result<Option<Option<PartialDate>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    match value {
        JsonValue::Null => Ok(Some(None)),
        JsonValue::String(v) => PartialDate::from_iso_str(&v)
            .map(|d| Some(Some(d)))
            .map_err(DeError::custom),
        other => Err(DeError::custom(format!(
            "expected date string or null, got {}",
            other
        ))),
    }
}

fn parse_year(s: &str) -> Option<Result<PartialDate, String>> {
    if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
        let year: i32 = s.parse().ok()?;
        Some(
            NaiveDate::from_ymd_opt(year, 1, 1)
                .ok_or(format!("invalid year '{}'", s))
                .map(|canonical| PartialDate {
                    canonical,
                    precision: DatePrecision::Year,
                }),
        )
    } else {
        None
    }
}

fn parse_month_iso(s: &str) -> Option<Result<PartialDate, String>> {
    let (year_str, month_str) = s.split_once('-')?;
    if year_str.len() != 4
        || !year_str.chars().all(|c| c.is_ascii_digit())
        || month_str.len() != 2
        || !month_str.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let year: i32 = year_str.parse().ok()?;
    let month: u32 = month_str.parse().ok()?;
    Some(
        NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or(format!("invalid month-year '{}'", s))
            .map(|canonical| PartialDate {
                canonical,
                precision: DatePrecision::Month,
            }),
    )
}

fn parse_day(s: &str) -> Option<Result<PartialDate, String>> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    if parts[0].len() != 4
        || !parts[0].chars().all(|c| c.is_ascii_digit())
        || parts[1].len() != 2
        || !parts[1].chars().all(|c| c.is_ascii_digit())
        || parts[2].len() != 2
        || !parts[2].chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    Some(
        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or(format!("invalid date '{}'", s))
            .map(|canonical| PartialDate {
                canonical,
                precision: DatePrecision::Day,
            }),
    )
}

fn parse_month_name(s: &str) -> Option<Result<PartialDate, String>> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 || parts[1].len() != 4 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let month_num = match parts[0].to_lowercase().as_str() {
        "jan" | "january" => 1,
        "feb" | "february" => 2,
        "mar" | "march" => 3,
        "apr" | "april" => 4,
        "may" => 5,
        "jun" | "june" => 6,
        "jul" | "july" => 7,
        "aug" | "august" => 8,
        "sep" | "september" => 9,
        "oct" | "october" => 10,
        "nov" | "november" => 11,
        "dec" | "december" => 12,
        _ => return None,
    };
    let year: i32 = parts[1].parse().ok()?;
    Some(
        NaiveDate::from_ymd_opt(year, month_num, 1)
            .ok_or(format!("invalid month-year '{}'", s))
            .map(|canonical| PartialDate {
                canonical,
                precision: DatePrecision::Month,
            }),
    )
}

fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("valid first day of next month")
        .pred_opt()
        .expect("valid last day of month")
}
