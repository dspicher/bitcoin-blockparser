use clap::{ArgMatches, Command};

use crate::blockchain::proto::tx::EvaluatedTx;
use crate::blockchain::proto::tx::TxOutpoint;
use crate::blockchain::proto::Hashed;
use crate::blockchain::proto::ToRaw;

use crate::blockchain::proto::block::Block;

pub mod balances;
pub mod csvdump;
pub mod opreturn;
pub mod simplestats;
pub mod unspentcsvdump;

/// Implement this trait for a custom Callback.
/// The parser ensures that the blocks arrive in the correct order.
/// At this stage the main chain is already determined and orphans/stales are removed.
pub trait Callback {
    /// Builds Command to specify callback name and required args,
    /// exits if some required args are missing.
    fn build_subcommand() -> Command
    where
        Self: Sized;

    /// Instantiates callback
    fn new(matches: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Gets called shortly before the blocks are parsed.
    fn on_start(&mut self, block_height: u64) -> anyhow::Result<()>;

    /// Gets called if a new block is available.
    fn on_block(&mut self, block: &Block, block_height: u64) -> anyhow::Result<()>;

    /// Gets called if the parser has finished and all blocks are handled
    fn on_complete(&mut self, block_height: u64) -> anyhow::Result<()>;

    /// Can be used to toggle whether the progress should be shown for specific callbacks or not
    fn show_progress(&self) -> bool {
        true
    }
}

pub struct UnspentValue {
    pub block_height: u64,
    pub value: u64,
    pub address: String,
}

pub struct UnspentsTracker(std::collections::HashMap<Vec<u8>, UnspentValue>);

impl UnspentsTracker {
    pub fn new() -> Self {
        Self(std::collections::HashMap::with_capacity(10000000))
    }
    /// Iterates over transaction inputs and removes spent outputs from HashMap.
    /// Returns the total number of processed inputs.
    pub fn remove_unspents(&mut self, tx: &Hashed<EvaluatedTx>) -> u64 {
        for input in &tx.value.inputs {
            let key = input.outpoint.to_bytes();
            self.0.remove(&key);
        }
        tx.value.in_count.value
    }

    /// Iterates over transaction outputs and adds valid unspents to HashMap.
    /// Returns the total number of valid outputs.
    pub fn insert_unspents(&mut self, tx: &Hashed<EvaluatedTx>, block_height: u64) -> u64 {
        let mut count = 0;
        for (i, output) in tx.value.outputs.iter().enumerate() {
            match &output.script.address {
                Some(address) => {
                    let unspent = UnspentValue {
                        block_height,
                        address: address.clone(),
                        value: output.out.value,
                    };

                    let key = TxOutpoint::new(tx.hash, i as u32).to_bytes();
                    self.0.insert(key, unspent);
                    count += 1;
                }
                None => {
                    log::debug!(
                        target: "callback", "Ignoring invalid utxo in: {} ({})",
                        &tx.hash,
                        output.script.pattern
                    );
                }
            }
        }
        count
    }
}
