use std::time::{Duration, Instant};

use crate::parser::chain::ChainStorage;
use crate::ParserOptions;

mod blkfile;
pub mod chain;
mod index;
pub mod reader;
pub mod types;

/// Small struct to hold statistics together
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
    chain_storage: ChainStorage, // Hash storage with the longest chain
    stats: WorkerStats,          // struct for thread management & statistics
    cur_height: u64,
}

impl BlockchainParser {
    /// Instantiates a new Parser.
    #[must_use]
    pub fn new(options: &ParserOptions, chain_storage: ChainStorage) -> Self {
        tracing::info!(target: "parser", "Parsing {} blockchain ...", options.coin.name);
        Self {
            chain_storage,
            stats: WorkerStats::new(options.range.start),
            cur_height: options.range.start,
        }
    }

    pub fn start(&mut self) {
        tracing::debug!(target: "parser", "Starting worker ...");

        self.on_start(self.cur_height);
        while let Some(header) = self.chain_storage.get_header(self.cur_height) {
            Self::on_header(&header, self.cur_height);
            let block = self.chain_storage.get_block(self.cur_height).unwrap();
            Self::on_block(&block, self.cur_height);
            self.print_progress(self.cur_height);
            self.cur_height += 1;
        }
        self.on_complete(self.cur_height.saturating_sub(1));
    }

    /// Returns number of remaining blocks
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

    fn on_block(_block: &bitcoin::Block, height: u64) {
        tracing::trace!(target: "parser", "on_block(height={}) called", height);
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
