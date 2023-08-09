use clap::ArgMatches;

pub mod balances;
pub mod opreturn;
pub mod simplestats;
pub mod unspentcsvdump;

/// Implement this trait for a custom Callback.
/// The parser ensures that the blocks arrive in the correct order.
/// At this stage the main chain is already determined and orphans/stales are removed.
pub trait Callback {
    /// Instantiates callback
    fn new(matches: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Gets called shortly before the blocks are parsed.
    fn on_start(&mut self, block_height: u64) -> anyhow::Result<()>;

    /// Gets called if a new block is available.
    fn on_block(&mut self, block: &bitcoin::Block, block_height: u64) -> anyhow::Result<()>;

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
    pub address: bitcoin::Address,
}

#[derive(Default)]
pub struct UnspentsTracker(
    std::collections::HashMap<bitcoin::blockdata::transaction::OutPoint, UnspentValue>,
);

impl UnspentsTracker {
    #[must_use]
    pub fn new() -> Self {
        Self(std::collections::HashMap::with_capacity(10_000_000))
    }
    /// Iterates over transaction inputs and removes spent outputs from HashMap.
    /// Returns the total number of processed inputs.
    pub fn remove_unspents(&mut self, tx: &bitcoin::Transaction) -> usize {
        for input in &tx.input {
            self.0.remove(&input.previous_output);
        }
        tx.input.len()
    }

    /// Iterates over transaction outputs and adds valid unspents to HashMap.
    /// Returns the total number of valid outputs.
    pub fn insert_unspents(&mut self, tx: &bitcoin::Transaction, block_height: u64) -> u64 {
        let mut count = 0;
        for (i, output) in tx.output.iter().enumerate() {
            match bitcoin::Address::from_script(&output.script_pubkey, bitcoin::Network::Bitcoin) {
                Ok(address) => {
                    let unspent = UnspentValue {
                        block_height,
                        address: address.clone(),
                        value: output.value,
                    };

                    let key = bitcoin::blockdata::transaction::OutPoint {
                        txid: tx.txid(),
                        vout: i as u32,
                    }; //TxOutTxOutpoint::new(tx.hash, i as u32).to_bytes();
                    self.0.insert(key, unspent);
                    count += 1;
                }
                Err(_) => {
                    tracing::debug!(
                        target: "callback", "Ignoring invalid utxo in: {}",
                        tx.txid(),
                    );
                }
            }
        }
        count
    }
}
