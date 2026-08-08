use infrastructure::{MIGRATION_LOCK_KEY, MIGRATIONS, run_migrations_once};

#[test]
fn migration_symbols_reexported_from_crate_root() {
    assert_eq!(MIGRATION_LOCK_KEY, 9_225_300);
    let _ = MIGRATIONS;
    let _ = run_migrations_once;
}
