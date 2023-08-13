use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::ArgMatches;

/// Dumps the UTXOs along with address in a csv file
pub struct UnspentCsvDump {
    dump_folder: PathBuf,
    writer: BufWriter<File>,
    unspents: super::UnspentsTracker,
    start_height: u64,
    tx_count: usize,
    in_count: usize,
    out_count: u64,
}

impl UnspentCsvDump {
    fn create_writer(cap: usize, path: PathBuf) -> anyhow::Result<BufWriter<File>> {
        Ok(BufWriter::with_capacity(cap, File::create(path)?))
    }
}

impl crate::callbacks::Callback for UnspentCsvDump {
    fn new(matches: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let dump_folder = &PathBuf::from(matches.get_one::<String>("dump-folder").unwrap());
        let cb = UnspentCsvDump {
            dump_folder: PathBuf::from(dump_folder),
            writer: UnspentCsvDump::create_writer(4_000_000, dump_folder.join("unspent.csv.tmp"))?,
            unspents: super::UnspentsTracker::new(),
            start_height: 0,
            tx_count: 0,
            in_count: 0,
            out_count: 0,
        };
        Ok(cb)
    }

    fn on_start(&mut self, block_height: u64) -> anyhow::Result<()> {
        self.start_height = block_height;
        tracing::info!(target: "callback", "Executing unspentcsvdump with dump folder: {} ...", &self.dump_folder.display());
        Ok(())
    }

    fn on_header(
        &mut self,
        _header: &bitcoin::blockdata::block::Header,
        _block_height: u64,
    ) -> anyhow::Result<()> {
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
            self.in_count += self.unspents.remove_unspents(tx);
            self.out_count += self.unspents.insert_unspents(tx, block_height);
        }
        self.tx_count += block.txdata.len();
        Ok(())
    }

    fn on_complete(&mut self, block_height: u64) -> anyhow::Result<()> {
        self.writer.write_all(
            format!(
                "{};{};{};{};{}\n",
                "txid", "indexOut", "height", "value", "address"
            )
            .as_bytes(),
        )?;
        for (key, value) in &self.unspents.0 {
            self.writer.write_all(
                format!(
                    "{};{};{};{};{}\n",
                    key.txid, key.vout, value.block_height, value.value, value.address
                )
                .as_bytes(),
            )?;
        }

        std::fs::rename(
            self.dump_folder.as_path().join("unspent.csv.tmp"),
            self.dump_folder.as_path().join(format!(
                "unspent-{}-{}.csv",
                self.start_height, block_height
            )),
        )?;

        tracing::info!(target: "callback", "Done.\nDumped blocks from height {} to {}:\n\
                                   \t-> transactions: {:9}\n\
                                   \t-> inputs:       {:9}\n\
                                   \t-> outputs:      {:9}",
             self.start_height, block_height, self.tx_count, self.in_count, self.out_count);
        Ok(())
    }
}
