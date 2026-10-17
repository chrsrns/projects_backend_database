use rocket::serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{ObjectBuilder, SchemaType, Type};
use utoipa::{PartialSchema, ToSchema};

/// Resume visibility. `private`: owner only. `unlisted`: readable by anyone
/// holding the UUID, absent from every non-owner listing. `public`: readable
/// and listed for everyone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum Visibility {
    Private,
    Unlisted,
    Public,
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Unlisted => "unlisted",
            Visibility::Public => "public",
        }
    }

    /// True when the row is reachable by id for a viewer who is not the owner.
    pub fn is_link_accessible(self) -> bool {
        matches!(self, Visibility::Unlisted | Visibility::Public)
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Private
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Visibility {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "private" => Ok(Visibility::Private),
            "unlisted" => Ok(Visibility::Unlisted),
            "public" => Ok(Visibility::Public),
            _ => Err(format!("invalid visibility '{}'", s)),
        }
    }
}

impl PartialSchema for Visibility {
    fn schema() -> RefOr<utoipa::openapi::schema::Schema> {
        ObjectBuilder::new()
            .schema_type(SchemaType::new(Type::String))
            .enum_values(Some(["private", "unlisted", "public"]))
            .description(Some("Resume visibility"))
            .into()
    }
}

impl ToSchema for Visibility {
    fn name() -> std::borrow::Cow<'static, str> {
        "Visibility".into()
    }
}
