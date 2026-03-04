use diesel::prelude::*;
use domain::models::Language;
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_accessible_resume},
};

pub fn list_languages(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<Language>, ApplicationError> {
    use domain::schema::languages::dsl as languages_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let mut items: Vec<Language> = match languages_dsl::languages
        .filter(languages_dsl::resume_id.eq(resume_id_value))
        .load::<Language>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    items.sort_by_key(|l| (l.display_order.unwrap_or(0), l.id));

    Ok(items)
}
