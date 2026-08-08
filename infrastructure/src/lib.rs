use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub mod migrations;

pub use migrations::{MIGRATION_LOCK_KEY, MIGRATIONS, run_migrations_once};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn run_in_transaction<T, F>(conn: &mut PgConnection, f: F) -> Result<T, diesel::result::Error>
where
    F: FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error>,
{
    conn.transaction::<T, _, _>(|conn| f(conn))
}
