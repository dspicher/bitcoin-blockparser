use clap::ArgMatches;

use crate::callbacks::Callback;

pub struct OpReturn {
    db: crate::db::Db,
}

impl Callback for OpReturn {
    fn new(_: &ArgMatches) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(OpReturn {
            db: crate::db::Db::open(),
        })
    }

    fn on_start(&mut self, _: u64) -> anyhow::Result<()> {
        tracing::info!(target: "callback", "Executing OpReturn ...");
        Ok(())
    }

    fn on_block(&mut self, block: &bitcoin::Block, block_height: u64) -> anyhow::Result<()> {
        for tx in &block.txdata {
            for (idx, out) in tx.output.iter().enumerate() {
                if out.script_pubkey.is_op_return() {
                    let script = out.script_pubkey.as_script();
                    if script.len() == 1 {
                        continue;
                    }
                    for inst in script.instructions().flatten() {
                        if let bitcoin::script::Instruction::PushBytes(data) = inst {
                            if let Ok(s) = String::from_utf8(data.as_bytes().into()) {
                                self.db.insert_opreturn(crate::db::Opreturn {
                                    height: block_height as i32,
                                    txid: tx.txid().to_string(),
                                    vout: idx as i32,
                                    message: s,
                                })?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn on_complete(&mut self, _: u64) -> anyhow::Result<()> {
        dbg!(self.db.opreturns(100, 0)?);
        Ok(())
    }
}
