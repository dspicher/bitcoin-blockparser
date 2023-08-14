use bitcoin::consensus::Decodable;

pub trait BlockchainRead: std::io::Read {
    fn read_block(&mut self) -> anyhow::Result<bitcoin::Block> {
        Ok(bitcoin::Block::consensus_decode(self).unwrap())
    }

    fn read_header(&mut self) -> anyhow::Result<bitcoin::blockdata::block::Header> {
        Ok(bitcoin::blockdata::block::Header::consensus_decode(self).unwrap())
    }
}

impl<R: std::io::Read + ?Sized> BlockchainRead for R {}
