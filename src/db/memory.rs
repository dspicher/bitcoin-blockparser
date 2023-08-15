use diesel_migrations::MigrationHarness;

pub type Pool =
    diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>>;

pub trait Managed {
    #[must_use]
    fn open(db_url: &str, migrations: diesel_migrations::EmbeddedMigrations) -> Pool {
        if db_url == ":memory:" {
            tracing::info!("opening in-memory database");
        } else {
            tracing::info!("opening database {db_url}");
        }
        let db = memdb_pool(db_url);
        create_tables(&db, migrations);
        db
    }
}

fn create_tables(pool: &Pool, migrations: diesel_migrations::EmbeddedMigrations) {
    let conn = &mut pool.get().unwrap();
    conn.run_pending_migrations(migrations)
        .expect("Could not apply database migration");
}

#[must_use]
fn memdb_pool(db_url: &str) -> Pool {
    let manager = diesel::r2d2::ConnectionManager::<diesel::sqlite::SqliteConnection>::new(db_url);
    let forever = Some(std::time::Duration::from_secs(u64::MAX));
    diesel::r2d2::Pool::builder()
        .idle_timeout(forever)
        .max_lifetime(forever)
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .expect("Problem creating connection pool")
}

impl Managed for Pool {}
