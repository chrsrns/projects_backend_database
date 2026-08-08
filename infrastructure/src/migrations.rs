use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;
use std::io;
use std::sync::OnceLock;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub const MIGRATION_LOCK_KEY: i64 = 9_225_300;

static MIGRATIONS_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

pub fn run_migrations_once() -> Result<(), io::Error> {
    MIGRATIONS_RESULT
        .get_or_init(|| run_migrations_inner())
        .clone()
        .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))
}

fn run_migrations_inner() -> Result<(), String> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

    let mut connection = PgConnection::establish(&database_url)
        .map_err(|e| format!("Error connecting to database: {}", e))?;

    let guard = acquire_advisory_lock(&mut connection, MIGRATION_LOCK_KEY)
        .map_err(|e| format!("acquire migrations advisory lock: {}", e))?;

    let result = guard
        .conn
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|e| format!("run pending migrations: {}", e));

    drop(guard);

    result
}

fn acquire_advisory_lock<'a>(
    conn: &'a mut PgConnection,
    key: i64,
) -> Result<AdvisoryLockGuard<'a>, diesel::result::Error> {
    diesel::sql_query("SELECT pg_advisory_lock($1)")
        .bind::<diesel::sql_types::BigInt, _>(key)
        .execute(conn)?;
    Ok(AdvisoryLockGuard { conn, key })
}

struct AdvisoryLockGuard<'a> {
    conn: &'a mut PgConnection,
    key: i64,
}

impl Drop for AdvisoryLockGuard<'_> {
    fn drop(&mut self) {
        let _ = diesel::sql_query("SELECT pg_advisory_unlock($1)")
            .bind::<diesel::sql_types::BigInt, _>(self.key)
            .execute(self.conn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_lock_key_is_9225300() {
        assert_eq!(MIGRATION_LOCK_KEY, 9_225_300);
    }

    #[test]
    fn run_migrations_once_is_ok() {
        let result = run_migrations_once();
        assert!(
            result.is_ok(),
            "run_migrations_once should succeed: {:?}",
            result
        );
    }

    #[test]
    fn run_migrations_once_caches_result() {
        let first = run_migrations_once();
        let second = run_migrations_once();
        assert!(first.is_ok(), "first call should succeed: {:?}", first);
        assert!(second.is_ok(), "second call should succeed: {:?}", second);
    }
}
