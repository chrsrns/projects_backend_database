use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::Deserialize;

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
