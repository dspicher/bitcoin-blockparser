use bitcoin::consensus::Decodable;

/// Trait for structured reading of blockchain data
pub trait BlockchainRead: std::io::Read {
    /// Reads a block as specified here: https://en.bitcoin.it/wiki/Protocol_specification#block
    fn read_block(&mut self) -> anyhow::Result<bitcoin::Block> {
        Ok(bitcoin::Block::consensus_decode(self).unwrap())
    }
}

/// All types that implement `Read` get methods defined in `BlockchainRead`
/// for free.
impl<R: std::io::Read + ?Sized> BlockchainRead for R {}
