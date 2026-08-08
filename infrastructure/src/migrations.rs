use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use std::io;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub const MIGRATION_LOCK_KEY: i64 = 9_225_300;

pub fn run_migrations_once() -> Result<(), io::Error> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "run_migrations_once not implemented",
    ))
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
