use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;

pub type Pool =
    diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>>;

pub trait Managed {
    #[must_use]
    fn open(migrations: diesel_migrations::EmbeddedMigrations) -> Pool {
        tracing::info!("opening in-memory database");
        let db = memdb_pool();
        create_tables(&db, migrations);
        db
    }
}

fn create_tables(pool: &Pool, migrations: diesel_migrations::EmbeddedMigrations) {
    let conn = &mut pool.get().unwrap();
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(conn)
        .unwrap();
    conn.run_pending_migrations(migrations)
        .expect("Could not apply database migration");
}

#[must_use]
fn memdb_pool() -> Pool {
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::sqlite::SqliteConnection>::new("opreturns.db");
    let forever = Some(std::time::Duration::from_secs(u64::MAX));
    diesel::r2d2::Pool::builder()
        .max_size(1)
        .idle_timeout(forever)
        .max_lifetime(forever)
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .expect("Problem creating connection pool")
}

impl Managed for Pool {}
