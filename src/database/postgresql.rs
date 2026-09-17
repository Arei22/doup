use diesel::Connection;
use diesel_async::AsyncPgConnection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::time::Duration;

const WAIT_BEFORE_RESTART: Duration = Duration::from_secs(10);

pub type PgPool = Pool<AsyncPgConnection>;
pub type PgPooled<'a> = PooledConnection<'a, AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn get_pool() -> PgPool {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
        std::env::var("DATABASE_URL").unwrap(),
        ManagerConfig::default(),
    );

    let max_pool_connections: u32 = std::env::var("MAX_POOL_CONNECTIONS")
        .unwrap_or_else(|_| "20".to_owned())
        .parse::<u32>()
        .unwrap_or(20);
    let min_reserved_pool_connections: Option<u32> = std::env::var("MIN_RESERVED_POOL_CONNECTIONS")
        .ok()
        .and_then(|val| val.parse::<u32>().ok());

    Pool::builder()
        .connection_timeout(WAIT_BEFORE_RESTART)
        .max_size(max_pool_connections)
        .min_idle(min_reserved_pool_connections)
        .build(manager)
        .await
        .unwrap_or_else(|error| {
            log::error!("{error}");
            std::process::exit(9);
        })
}

pub async fn run_migration() {
    tokio::task::spawn_blocking(move || {
        let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(
            &std::env::var("DATABASE_URL").unwrap(),
        )
        .unwrap_or_else(|error| {
            log::error!("Error when connecting to database to deploy migrations: {error}",);
            std::process::exit(1);
        });
        conn.run_pending_migrations(MIGRATIONS)
            .unwrap_or_else(|error| {
                log::error!("Error when deploying migrations: {error}");
                std::process::exit(1);
            });
    })
    .await
    .unwrap_or_else(|error| {
        log::error!("Error when deploying migrations: {error}");
        std::process::exit(1);
    });
}
