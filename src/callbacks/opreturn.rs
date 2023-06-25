use clap::{ArgMatches, Command};

use crate::callbacks::Callback;

#[derive(Default)]
pub struct OpReturn;

impl Callback for OpReturn {
    fn build_subcommand() -> Command
    where
        Self: Sized,
    {
        Command::new("opreturn")
            .about("Shows embedded OP_RETURN data that is representable as UTF8")
            .version("0.1")
            .author("gcarq <egger.m@protonmail.com>")
    }

    fn new(_: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(OpReturn::default())
    }

    fn on_start(&mut self, _: u64) -> anyhow::Result<()> {
        log::info!(target: "callback", "Executing OpReturn ...");
        Ok(())
    }

    fn on_block(&mut self, block: &bitcoin::Block, block_height: u64) -> anyhow::Result<()> {
        for tx in &block.txdata {
            for out in tx.output.iter() {
                if out.script_pubkey.is_op_return() {
                    let script = out.script_pubkey.as_script();
                    if script.len() == 1 {
                        continue;
                    }
                    for inst in script.instructions() {
                        if let Ok(bitcoin::script::Instruction::PushBytes(data)) = inst {
                            if let Ok(s) = String::from_utf8(data.as_bytes().into()) {
                                println!(
                                    "height: {: <9} txid: {}    message: {}",
                                    block_height,
                                    &tx.txid(),
                                    s
                                );
                            } else {
                                println!(
                                    "height: {: <9} txid: {}    data: {}",
                                    block_height,
                                    &tx.txid(),
                                    hex::encode(data.as_bytes())
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn on_complete(&mut self, _: u64) -> anyhow::Result<()> {
        Ok(())
    }

    fn show_progress(&self) -> bool {
        false
    }
}
