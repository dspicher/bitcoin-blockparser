use self::memory::Managed;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use schema::blocks;

mod memory;
pub mod schema;

#[derive(Clone)]
pub struct Db {
    pub pool: memory::Pool,
}

#[derive(Debug, diesel::Selectable, diesel::Queryable, diesel::Insertable)]
pub struct Block {
    pub height: i32,
    pub version: i32,
    pub time: i32,
    pub encoded_target: i32,
    pub nonce: i64,
    pub tx_count: i32,
    pub size: i32,
    pub weight: i64,
    pub turnover: i64,
}

const MIGRATIONS: diesel_migrations::EmbeddedMigrations = diesel_migrations::embed_migrations!();

impl Db {
    #[must_use]
    pub fn open(db_url: &str) -> Self {
        Self {
            pool: memory::Pool::open(db_url, MIGRATIONS),
        }
    }

    pub fn insert_blocks(&self, blocks: Vec<Block>) -> anyhow::Result<usize> {
        Ok(diesel::insert_into(blocks::table)
            .values(blocks)
            .execute(&mut self.pool.get()?)?)
    }

    pub fn block(&self, height: i32) -> anyhow::Result<Block> {
        Ok(blocks::table
            .select(Block::as_select())
            .filter(blocks::height.eq(height))
            .get_result(&mut self.pool.get()?)?)
    }

    pub fn blocks_count(&self) -> anyhow::Result<i64> {
        Ok(blocks::table.count().get_result(&mut self.pool.get()?)?)
    }
}
