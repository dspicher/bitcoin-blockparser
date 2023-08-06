use bitcoin::hashes::{sha256d, Hash};
use std::collections::HashMap;
use std::io::Write;

use clap::{ArgMatches, Command};

use crate::callbacks::Callback;

pub struct SimpleStats {
    n_valid_blocks: u64,
    block_sizes: Vec<usize>,

    n_tx: usize,
    n_tx_inputs: usize,
    n_tx_outputs: usize,
    n_tx_total_fee: u64,
    n_tx_total_volume: u64,

    /// Biggest value transaction (value, height, txid)
    tx_biggest_value: (u64, u64, bitcoin::Txid),
    /// Biggest size transaction (size, height, txid)
    tx_biggest_size: (usize, u64, bitcoin::Txid),
    /// Contains transaction type count
    n_tx_types: HashMap<bitcoin::AddressType, u64>,
    /// First occurence of transaction type
    /// (block_height, txid, index)
    tx_first_occs: HashMap<bitcoin::AddressType, (u64, bitcoin::Txid, usize)>,

    /// Time stats
    t_between_blocks: Vec<u32>,
    last_timestamp: u32,
}

impl Default for SimpleStats {
    fn default() -> Self {
        SimpleStats {
            n_valid_blocks: 0,
            block_sizes: vec![],
            n_tx: 0,
            n_tx_inputs: 0,
            n_tx_outputs: 0,
            n_tx_total_fee: 0,
            n_tx_total_volume: 0,
            tx_biggest_value: (0, 0, sha256d::Hash::all_zeros().into()),
            tx_biggest_size: (0, 0, sha256d::Hash::all_zeros().into()),
            n_tx_types: HashMap::new(),
            tx_first_occs: HashMap::new(),
            t_between_blocks: vec![],
            last_timestamp: 0,
        }
    }
}

impl SimpleStats {
    /// Saves transaction pattern with txid of first occurence
    fn process_tx_pattern(
        &mut self,
        address_type: bitcoin::AddressType,
        block_height: u64,
        txid: bitcoin::Txid,
        index: usize,
    ) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.n_tx_types.entry(address_type) {
            e.insert(1);
            self.tx_first_occs
                .insert(address_type, (block_height, txid, index));
        } else {
            let counter = self.n_tx_types.entry(address_type).or_insert(1);
            *counter += 1;
        }
    }

    fn print_simple_stats(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        writeln!(buffer, "SimpleStats:")?;
        writeln!(buffer, "   -> valid blocks:\t\t{}", self.n_valid_blocks)?;
        writeln!(buffer, "   -> total transactions:\t{}", self.n_tx)?;
        writeln!(buffer, "   -> total tx inputs:\t\t{}", self.n_tx_inputs)?;
        writeln!(buffer, "   -> total tx outputs:\t\t{}", self.n_tx_outputs)?;
        writeln!(
            buffer,
            "   -> total tx fees:\t\t{:.8} ({} units)",
            self.n_tx_total_fee as f64 * 1E-8,
            self.n_tx_total_fee
        )?;
        writeln!(
            buffer,
            "   -> total volume:\t\t{:.8} ({} units)",
            self.n_tx_total_volume as f64 * 1E-8,
            self.n_tx_total_volume
        )?;
        Ok(())
    }

    fn print_averages(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        writeln!(buffer, "Averages:")?;
        writeln!(
            buffer,
            "   -> avg block size:\t\t{:.2} KiB",
            self.block_sizes.iter().sum::<usize>() as f64
                / (1024.00 * self.block_sizes.len() as f64)
        )?;
        writeln!(
            buffer,
            "   -> avg time between blocks:\t{:.2} (minutes)",
            f64::from(self.t_between_blocks.iter().sum::<u32>())
                / (60.00 * self.t_between_blocks.len() as f64)
        )?;
        writeln!(
            buffer,
            "   -> avg txs per block:\t{:.2}",
            self.n_tx as f64 / self.n_valid_blocks as f64
        )?;
        writeln!(
            buffer,
            "   -> avg inputs per tx:\t{:.2}",
            self.n_tx_inputs as f64 / self.n_tx as f64
        )?;
        writeln!(
            buffer,
            "   -> avg outputs per tx:\t{:.2}",
            self.n_tx_outputs as f64 / self.n_tx as f64
        )?;
        writeln!(
            buffer,
            "   -> avg value per output:\t{:.2}",
            self.n_tx_total_volume as f64 / self.n_tx_outputs as f64 * 1E-8
        )?;
        Ok(())
    }

    fn print_unusual_transactions(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        let (value, height, txid) = self.tx_biggest_value;
        writeln!(
            buffer,
            "   -> biggest value tx:\t\t{:.8} ({} units)",
            value as f64 * 1E-8,
            value
        )?;
        writeln!(
            buffer,
            "        seen in block #{}, txid: {}\n",
            height, &txid
        )?;
        let (value, height, txid) = self.tx_biggest_size;
        writeln!(buffer, "   -> biggest size tx:\t\t{value} bytes",)?;
        writeln!(
            buffer,
            "        seen in block #{}, txid: {}\n",
            height, &txid
        )?;
        Ok(())
    }

    fn print_transaction_types(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        writeln!(buffer, "Transaction Types:")?;
        for (pattern, count) in &self.n_tx_types {
            writeln!(
                buffer,
                "   -> {:?}: {} ({:.2}%)",
                pattern,
                count,
                (*count as f64 / self.n_tx_outputs as f64) * 100.00
            )?;

            let pos = self.tx_first_occs.get(pattern).unwrap();
            writeln!(
                buffer,
                "        first seen in block #{}, txid: {}\n",
                pos.0, &pos.1
            )?;
        }
        Ok(())
    }
}

impl Callback for SimpleStats {
    fn build_subcommand() -> Command
    where
        Self: Sized,
    {
        Command::new("simplestats")
            .about("Shows various Blockchain stats")
            .version("0.1")
            .author("gcarq <egger.m@protonmail.com>")
    }

    fn new(_: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(SimpleStats::default())
    }

    fn on_start(&mut self, _: u64) -> anyhow::Result<()> {
        log::info!(target: "callback", "Executing simplestats ...");
        Ok(())
    }

    fn on_block(&mut self, block: &bitcoin::Block, block_height: u64) -> anyhow::Result<()> {
        self.n_valid_blocks += 1;
        self.n_tx += block.txdata.len();
        self.block_sizes.push(block.size());

        for tx in &block.txdata {
            // Collect fee rewards
            if tx.is_coin_base() {
                self.n_tx_total_fee += tx.output[0].value;
                // .checked_sub(block::get_base_reward(block_height))
                // .unwrap_or_default();
            }

            self.n_tx_inputs += tx.input.len();
            self.n_tx_outputs += tx.output.len();

            let mut tx_value = 0;
            for (i, o) in tx.output.iter().enumerate() {
                if let Ok(addr) =
                    bitcoin::Address::from_script(&o.script_pubkey, bitcoin::Network::Bitcoin)
                {
                    if let Some(t) = addr.address_type() {
                        self.process_tx_pattern(t, block_height, tx.txid(), i);
                    }
                }
                tx_value += o.value;
            }
            // Calculate and save biggest value transaction
            if tx_value > self.tx_biggest_value.0 {
                self.tx_biggest_value = (tx_value, block_height, tx.txid());
            }

            self.n_tx_total_volume += tx_value;

            // Calculate and save biggest size transaction
            let tx_size = tx.size();
            if tx_size > self.tx_biggest_size.0 {
                self.tx_biggest_size = (tx_size, block_height, tx.txid());
            }
        }

        // Save time between blocks
        if self.last_timestamp > 0 {
            let diff = block
                .header
                .time
                .checked_sub(self.last_timestamp)
                .unwrap_or_default();
            self.t_between_blocks.push(diff);
        }
        self.last_timestamp = block.header.time;
        Ok(())
    }

    fn on_complete(&mut self, _: u64) -> anyhow::Result<()> {
        let mut buffer = Vec::with_capacity(4096);
        self.print_simple_stats(&mut buffer)?;
        writeln!(&mut buffer)?;
        self.print_unusual_transactions(&mut buffer)?;
        self.print_averages(&mut buffer)?;
        writeln!(&mut buffer)?;
        self.print_transaction_types(&mut buffer)?;
        log::info!(target: "simplestats", "\n\n{}", String::from_utf8_lossy(&buffer));
        Ok(())
    }
}
