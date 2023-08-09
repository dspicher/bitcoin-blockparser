use std::time::{Duration, Instant};

use diesel::connection::TransactionManager;
use diesel::RunQueryDsl;

use crate::parser::chain::ChainStorage;
use crate::ParserOptions;

mod blkfile;
pub mod chain;
mod index;
pub mod reader;
pub mod types;

struct WorkerStats {
    pub started_at: Instant,
    pub last_log: Instant,
    pub last_height: u64,
}

impl WorkerStats {
    fn new(start_range: u64) -> Self {
        Self {
            started_at: Instant::now(),
            last_log: Instant::now(),
            last_height: start_range,
        }
    }
}

pub struct BlockchainParser {
    chain_storage: ChainStorage,
    stats: WorkerStats,
    cur_height: u64,
    db: crate::db::Db,
}

impl BlockchainParser {
    #[must_use]
    pub fn new(options: &ParserOptions, chain_storage: ChainStorage) -> Self {
        tracing::info!(target: "parser", "Parsing {} blockchain ...", options.coin.name);
        Self {
            chain_storage,
            stats: WorkerStats::new(options.range.start),
            cur_height: options.range.start,
            db: crate::db::Db::open(&options.db_url),
        }
    }

    #[must_use]
    pub fn db(&self) -> &crate::db::Db {
        &self.db
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        tracing::debug!(target: "parser", "Starting worker ...");

        self.on_start(self.cur_height);
        let mut conn = self.db.pool.get()?;
        diesel::r2d2::PoolTransactionManager::begin_transaction(&mut conn)?;
        while let Some(header) = self.chain_storage.get_header(self.cur_height) {
            Self::on_header(&header, self.cur_height);
            let block = self.chain_storage.get_block(self.cur_height).unwrap();
            self.on_block(&block, self.cur_height, &mut conn)?;
            self.print_progress(self.cur_height);
            self.cur_height += 1;

            if self.cur_height % 1000 == 0 {
                diesel::r2d2::PoolTransactionManager::commit_transaction(&mut conn)?;
                drop(conn);
                conn = self.db.pool.get()?;
                diesel::r2d2::PoolTransactionManager::begin_transaction(&mut conn)?;
            }
        }
        diesel::r2d2::PoolTransactionManager::commit_transaction(&mut conn)?;
        drop(conn);
        self.on_complete(self.cur_height.saturating_sub(1));
        Ok(())
    }

    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.chain_storage
            .max_height()
            .saturating_sub(self.cur_height)
    }

    fn on_start(&mut self, height: u64) {
        let now = Instant::now();
        self.stats.started_at = now;
        self.stats.last_log = now;
        tracing::info!(target: "parser", "Processing blocks starting from height {} ...", height);
        tracing::trace!(target: "parser", "on_start() called");
    }

    fn on_header(_header: &bitcoin::blockdata::block::Header, height: u64) {
        tracing::trace!(target: "parser", "on_header(height={}) called", height);
    }

    fn on_block(
        &mut self,
        block: &bitcoin::Block,
        height: u64,
        conn: &mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>,
        >,
    ) -> anyhow::Result<()> {
        tracing::trace!(target: "parser", "on_block(height={}) called", height);
        diesel::insert_into(crate::db::schema::blocks::table)
            .values(crate::db::Block {
                height: self.cur_height.try_into()?,
                version: block.header.version.to_consensus(),
                time: block.header.time.try_into()?,
                encoded_target: block.header.bits.to_consensus().try_into()?,
                nonce: block.header.nonce.try_into()?,
                tx_count: block.txdata.len().try_into()?,
                size: block.size().try_into()?,
                weight: block.weight().to_wu().try_into()?,
            })
            .execute(conn)?;
        Ok(())
    }

    fn on_complete(&mut self, height: u64) {
        tracing::info!(target: "parser", "Done. Processed blocks up to height {} in {:.2} minutes.",
        height, self.stats.started_at.elapsed().as_secs_f32() / 60.0);

        tracing::trace!(target: "parser", "on_complete() called");
    }

    fn print_progress(&mut self, height: u64) {
        let measure_frame = 10;
        let now = Instant::now();
        let blocks_speed = (height - self.stats.last_height) / measure_frame;

        if now - self.stats.last_log > Duration::from_secs(measure_frame) {
            tracing::info!(target: "parser", "Status: {:7} Blocks processed. (remaining: {:7}, speed: {:5.2} blocks/s)",
              height, self.remaining(), blocks_speed);
            self.stats.last_log = now;
            self.stats.last_height = height;
        }
    }
}
