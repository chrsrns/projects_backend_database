use rocket::serde::Deserialize;
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;

mod auth;
mod education;
mod frameworks;
mod languages;
mod parsed_resume;
mod partial_date;
mod portfolio;
mod resume;
mod resume_view;
mod skills;
mod work_experience;

pub use auth::*;
pub use education::*;
pub use frameworks::*;
pub use languages::*;
pub use parsed_resume::*;
pub use partial_date::*;
pub use portfolio::*;
pub use resume::*;
pub use resume_view::*;
pub use skills::*;
pub use work_experience::*;

/// Decodes a nullable string patch field: an absent key stays `None`, an
/// explicit `null` becomes `Some(None)` (store NULL), and a string becomes
/// `Some(Some(_))`. Non-string scalars are rejected.
pub fn deserialize_optional_nullable_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    match value {
        JsonValue::Null => Ok(Some(None)),
        JsonValue::String(v) => Ok(Some(Some(v))),
        other => Err(DeError::custom(format!(
            "expected string or null, got {}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateResume;

    #[test]
    fn patch_string_field_null_decodes_to_explicit_null() {
        let dto: UpdateResume = serde_json::from_str(r#"{"video": null}"#).unwrap();
        assert_eq!(dto.video, Some(None));
    }

    #[test]
    fn patch_string_field_string_decodes_to_value() {
        let dto: UpdateResume = serde_json::from_str(r#"{"video": "https://x/v.mp4"}"#).unwrap();
        assert_eq!(dto.video, Some(Some("https://x/v.mp4".to_string())));
    }

    #[test]
    fn patch_string_field_non_string_is_rejected() {
        let result = serde_json::from_str::<UpdateResume>(r#"{"video": 5}"#);
        assert!(result.is_err());
    }

    #[test]
    fn patch_string_field_absent_decodes_to_no_change() {
        let dto: UpdateResume = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(dto.video, None);
        assert_eq!(dto.company_name, None);
    }
}
