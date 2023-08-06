use crate::parser::blkfile::BlkFile;
use crate::parser::index::ChainIndex;
use crate::parser::types::CoinType;
use crate::ParserOptions;

/// Manages the index and data of longest valid chain
pub struct ChainStorage {
    chain_index: ChainIndex,
    blk_files: std::collections::HashMap<u64, BlkFile>, // maps blk_index to BlkFile
    coin: CoinType,
    verify: bool,
}

impl ChainStorage {
    pub fn new(options: &ParserOptions) -> anyhow::Result<Self> {
        Ok(Self {
            chain_index: ChainIndex::new(options)?,
            blk_files: BlkFile::from_path(options.blockchain_dir.as_path())?,
            coin: options.coin.clone(),
            verify: options.verify,
        })
    }

    /// Returns the next block and its height
    #[must_use]
    pub fn get_block(&mut self, height: u64) -> Option<bitcoin::Block> {
        // Read block
        let block_meta = self.chain_index.get(height)?;
        let blk_file = self.blk_files.get_mut(&block_meta.blk_index)?;
        let block = blk_file.read_block(block_meta.data_offset).ok()?;

        // Check if blk file can be closed
        if height == self.chain_index.max_height_by_blk(block_meta.blk_index) {
            blk_file.close();
        }

        if self.verify {
            self.verify(&block, height).unwrap();
        }

        Some(block)
    }

    /// Verifies the given block in a chain.
    /// Panics if not valid
    fn verify(&self, block: &bitcoin::Block, height: u64) -> anyhow::Result<()> {
        assert!(block.check_merkle_root());
        if height == 0 {
            if block.header.block_hash().as_raw_hash() != &self.coin.genesis_hash {
                anyhow::bail!(
                    "Genesis block hash doesn't match!\n  -> expected: {}\n  -> got: {}\n",
                    &self.coin.genesis_hash,
                    &block.header.block_hash(),
                );
            }
        } else {
            let prev_hash = self
                .chain_index
                .get(height - 1)
                .expect("unable to fetch prev block in chain index")
                .block_hash;
            if block.header.prev_blockhash.as_raw_hash() != &prev_hash {
                anyhow::bail!(
                    "prev_hash for block {} doesn't match!\n  -> expected: {}\n  -> got: {}\n",
                    &block.header.block_hash(),
                    &block.header.prev_blockhash,
                    &prev_hash
                );
            }
        }
        Ok(())
    }

    pub(crate) fn max_height(&self) -> u64 {
        self.chain_index.max_height()
    }
}
