use diesel::ConnectionResult;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenvy::dotenv;
use std::env;
use std::sync::OnceLock;

pub mod migrations;

pub use migrations::{MIGRATION_LOCK_KEY, MIGRATIONS, run_migrations_once};

/// Maximum connections the request-path pool keeps open. When every
/// connection is checked out, acquisition blocks until one is returned
/// or the pool's connection timeout elapses.
pub const DB_POOL_MAX_SIZE: u32 = 10;

pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

fn database_url() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set.")
}

pub fn db_pool() -> &'static Pool<ConnectionManager<PgConnection>> {
    static POOL: OnceLock<Pool<ConnectionManager<PgConnection>>> = OnceLock::new();
    POOL.get_or_init(|| {
        Pool::builder()
            .max_size(DB_POOL_MAX_SIZE)
            .build(ConnectionManager::new(database_url()))
            .expect("failed to build the database connection pool")
    })
}

/// Checks out a pooled request-path connection. Blocks up to the pool's
/// connection timeout when all connections are in use, then panics on
/// acquire failure. Session-scoped advisory locks and migrations must
/// use `establish_dedicated_connection` instead so they never pin a
/// pool slot.
pub fn establish_connection() -> DbConnection {
    db_pool()
        .get()
        .expect("failed to acquire a connection from the database pool")
}

/// Opens a dedicated (unpooled) connection. Fallible so callers that
/// must react to a refused connection — migrations and session-scoped
/// advisory locks — can handle the error.
pub fn establish_dedicated_connection() -> ConnectionResult<PgConnection> {
    PgConnection::establish(&database_url())
}

pub fn run_in_transaction<T, F>(conn: &mut DbConnection, f: F) -> Result<T, diesel::result::Error>
where
    F: FnOnce(&mut DbConnection) -> Result<T, diesel::result::Error>,
{
    conn.transaction::<T, _, _>(|conn| f(conn))
}
