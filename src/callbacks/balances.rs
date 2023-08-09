use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};

use crate::callbacks::Callback;

/// Dumps all addresses with non-zero balance in a csv file
pub struct Balances {
    dump_folder: PathBuf,
    writer: BufWriter<File>,
    unspents: super::UnspentsTracker,
    start_height: u64,
    end_height: u64,
}

impl Balances {
    fn create_writer(cap: usize, path: PathBuf) -> anyhow::Result<BufWriter<File>> {
        Ok(BufWriter::with_capacity(cap, File::create(path)?))
    }
}

impl Callback for Balances {
    fn build_subcommand() -> Command
    where
        Self: Sized,
    {
        Command::new("balances")
            .about("Dumps all addresses with non-zero balance to CSV file")
            .arg(
                Arg::new("dump-folder")
                    .help("Folder to store csv file")
                    .index(1)
                    .required(true),
            )
    }

    fn new(matches: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let dump_folder = &PathBuf::from(matches.get_one::<String>("dump-folder").unwrap());
        let cb = Balances {
            dump_folder: PathBuf::from(dump_folder),
            writer: Balances::create_writer(4_000_000, dump_folder.join("balances.csv.tmp"))?,
            unspents: super::UnspentsTracker::new(),
            start_height: 0,
            end_height: 0,
        };
        Ok(cb)
    }

    fn on_start(&mut self, block_height: u64) -> anyhow::Result<()> {
        self.start_height = block_height;
        tracing::info!(target: "callback", "Executing balances with dump folder: {} ...", &self.dump_folder.display());
        Ok(())
    }

    /// For each transaction in the block
    ///   1. apply input transactions (remove (TxID == prevTxIDOut and prevOutID == spentOutID))
    ///   2. apply output transactions (add (TxID + curOutID -> HashMapVal))
    /// For each address, retain:
    ///   * block height as "last modified"
    ///   * output_val
    ///   * address
    fn on_block(&mut self, block: &bitcoin::Block, block_height: u64) -> anyhow::Result<()> {
        for tx in &block.txdata {
            self.unspents.remove_unspents(tx);
            self.unspents.insert_unspents(tx, block_height);
        }
        Ok(())
    }

    fn on_complete(&mut self, block_height: u64) -> anyhow::Result<()> {
        self.end_height = block_height;

        self.writer
            .write_all(format!("{};{}\n", "address", "balance").as_bytes())?;

        // Collect balances for each address
        let mut balances = std::collections::HashMap::new();
        for unspent in self.unspents.0.values() {
            let entry = balances.entry(&unspent.address).or_insert(0);
            *entry += unspent.value;
        }

        for (address, balance) in &balances {
            self.writer
                .write_all(format!("{address};{balance}\n").as_bytes())?;
        }

        std::fs::rename(
            self.dump_folder.as_path().join("balances.csv.tmp"),
            self.dump_folder.as_path().join(format!(
                "balances-{}-{}.csv",
                self.start_height, self.end_height
            )),
        )
        .expect("Unable to rename tmp file!");

        tracing::info!(target: "callback", "Done.\nDumped {} addresses.", balances.len());
        Ok(())
    }
}
