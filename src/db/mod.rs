use self::mem::Managed;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use schema::opreturns;

mod mem;
mod schema;

#[derive(Clone)]
pub struct Db {
    pool: mem::Pool,
}

#[derive(Debug, diesel::Selectable, diesel::Queryable, diesel::Insertable)]
pub struct Opreturn {
    pub height: i32,
    pub txid: String,
    pub vout: i32,
    pub message: String,
}

const MIGRATIONS: diesel_migrations::EmbeddedMigrations = diesel_migrations::embed_migrations!();

impl Db {
    #[must_use]
    pub fn open() -> Self {
        Self {
            pool: mem::Pool::open(MIGRATIONS),
        }
    }

    pub fn insert_opreturn(&self, opreturn: Opreturn) -> anyhow::Result<usize> {
        Ok(diesel::insert_into(opreturns::table)
            .values(opreturn)
            .execute(&mut self.pool.get()?)?)
    }

    pub fn opreturns(&self, limit: i32, offset: i32) -> anyhow::Result<Vec<Opreturn>> {
        Ok(opreturns::table
            .select(Opreturn::as_select())
            .order_by(opreturns::height.asc())
            .limit(limit.into())
            .offset(offset.into())
            .get_results(&mut self.pool.get()?)?)
    }

    pub fn opreturn_count(&self) -> anyhow::Result<i64> {
        Ok(opreturns::table
            .count()
            .get_result(&mut self.pool.get()?)?)
    }
}
