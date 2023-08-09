mod common;

fn storage() -> bitcoin_blockparser::parser::chain::ChainStorage {
    common::storage("testnet3", 120)
}

fn parser() -> bitcoin_blockparser::parser::BlockchainParser {
    common::parser("testnet3", 120)
}

#[test]
fn test_blockdata_parsing() {
    let mut storage = storage();
    let genesis = storage.get_block(0).unwrap();
    assert_eq!(
        genesis,
        bitcoin::blockdata::constants::genesis_block(bitcoin::network::constants::Network::Testnet)
    );
    assert_eq!(
        genesis.block_hash().to_string(),
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943"
    );
    for height in 0..=120 {
        let block = storage.get_block(height).unwrap();
        assert_eq!(block.txdata.len(), 1);
    }
}

#[test]
fn test_genesis_header() {
    let mut storage = storage();
    let header = storage.get_header(0).unwrap();
    assert_eq!(
        header,
        bitcoin::blockdata::constants::genesis_block(bitcoin::network::constants::Network::Testnet)
            .header
    );
}

#[test]
fn test_blocks_db() {
    let mut parser = parser();
    parser.start().unwrap();
    assert_eq!(parser.db().blocks_count().unwrap(), 121);
}
