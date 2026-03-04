use diesel::prelude::*;
use domain::models::{Framework, Language};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_accessible_resume},
};

pub fn list_frameworks(
    resume_id_value: i32,
    language_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<Framework>, ApplicationError> {
    use domain::schema::frameworks::dsl as frameworks_dsl;
    use domain::schema::languages::dsl as languages_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let language: Language = match languages_dsl::languages
        .filter(languages_dsl::id.eq(language_id_value))
        .filter(languages_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(l) => l,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    let mut items: Vec<Framework> = match frameworks_dsl::frameworks
        .filter(frameworks_dsl::language_id.eq(language.id))
        .load::<Framework>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    items.sort_by_key(|f| (f.display_order.unwrap_or(0), f.id));

    Ok(items)
}
