use std::time::{Duration, Instant};

use crate::callbacks::Callback;
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
    pub measure_frame: Duration,
}

impl WorkerStats {
    fn new(start_range: u64) -> Self {
        Self {
            started_at: Instant::now(),
            last_log: Instant::now(),
            last_height: start_range,
            measure_frame: Duration::from_secs(10),
        }
    }
}

pub struct BlockchainParser {
    chain_storage: ChainStorage, // Hash storage with the longest chain
    stats: WorkerStats,          // struct for thread management & statistics
    callback: Box<dyn Callback>,
    cur_height: u64,
}

impl BlockchainParser {
    /// Instantiates a new Parser.
    #[must_use]
    pub fn new(options: ParserOptions, chain_storage: ChainStorage) -> Self {
        tracing::info!(target: "parser", "Parsing {} blockchain ...", options.coin.name);
        Self {
            chain_storage,
            stats: WorkerStats::new(options.range.start),
            callback: options.callback,
            cur_height: options.range.start,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        tracing::debug!(target: "parser", "Starting worker ...");

        self.on_start(self.cur_height)?;
        while let Some(block) = self.chain_storage.get_block(self.cur_height) {
            self.on_block(&block, self.cur_height)?;
            self.cur_height += 1;
        }
        self.on_complete(self.cur_height.saturating_sub(1))
    }

    /// Returns number of remaining blocks
    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.chain_storage
            .max_height()
            .saturating_sub(self.cur_height)
    }

    /// Triggers the on_start() callback and initializes state.
    fn on_start(&mut self, height: u64) -> anyhow::Result<()> {
        let now = Instant::now();
        self.stats.started_at = now;
        self.stats.last_log = now;
        tracing::info!(target: "parser", "Processing blocks starting from height {} ...", height);
        self.callback.on_start(height)?;
        tracing::trace!(target: "parser", "on_start() called");
        Ok(())
    }

    /// Triggers the on_block() callback and updates statistics.
    fn on_block(&mut self, block: &bitcoin::Block, height: u64) -> anyhow::Result<()> {
        self.callback.on_block(block, height)?;
        tracing::trace!(target: "parser", "on_block(height={}) called", height);
        self.print_progress(height);
        Ok(())
    }

    /// Triggers the on_complete() callback and updates statistics.
    fn on_complete(&mut self, height: u64) -> anyhow::Result<()> {
        tracing::info!(target: "parser", "Done. Processed blocks up to height {} in {:.2} minutes.",
        height, self.stats.started_at.elapsed().as_secs_f32() / 60.0);

        self.callback.on_complete(height)?;
        tracing::trace!(target: "parser", "on_complete() called");
        Ok(())
    }

    fn print_progress(&mut self, height: u64) {
        let now = Instant::now();
        let blocks_speed = (height - self.stats.last_height) / self.stats.measure_frame.as_secs();

        if now - self.stats.last_log > self.stats.measure_frame {
            tracing::info!(target: "parser", "Status: {:7} Blocks processed. (remaining: {:7}, speed: {:5.2} blocks/s)",
              height, self.remaining(), blocks_speed);
            self.stats.last_log = now;
            self.stats.last_height = height;
        }
    }
}
