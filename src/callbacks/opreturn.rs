use clap::ArgMatches;

use crate::callbacks::Callback;

#[derive(Default)]
pub struct OpReturn;

impl Callback for OpReturn {
    fn new(_: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(OpReturn)
    }

    fn on_start(&mut self, _: u64) -> anyhow::Result<()> {
        tracing::info!(target: "callback", "Executing OpReturn ...");
        Ok(())
    }

    fn on_block(&mut self, block: &bitcoin::Block, block_height: u64) -> anyhow::Result<()> {
        for tx in &block.txdata {
            for out in &tx.output {
                if out.script_pubkey.is_op_return() {
                    let script = out.script_pubkey.as_script();
                    if script.len() == 1 {
                        continue;
                    }
                    for inst in script.instructions().flatten() {
                        if let bitcoin::script::Instruction::PushBytes(data) = inst {
                            if let Ok(s) = String::from_utf8(data.as_bytes().into()) {
                                println!(
                                    "height: {: <9} txid: {}    message: {}",
                                    block_height,
                                    &tx.txid(),
                                    s
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
}
