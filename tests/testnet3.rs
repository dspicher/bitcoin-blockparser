mod common;

static STORAGE: once_cell::sync::Lazy<
    std::sync::Mutex<bitcoin_blockparser::parser::chain::ChainStorage>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(common::storage("testnet3", 120)));

#[test]
fn test_blockdata_parsing() {
    let genesis = STORAGE.lock().unwrap().get_block(0).unwrap();
    assert_eq!(
        genesis,
        bitcoin::blockdata::constants::genesis_block(bitcoin::network::constants::Network::Testnet)
    );
    assert_eq!(
        genesis.block_hash().to_string(),
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943"
    );
    for height in 0..=120 {
        let block = STORAGE.lock().unwrap().get_block(height).unwrap();
        assert_eq!(block.txdata.len(), 1);
    }
}

#[test]
fn test_genesis_header() {
    let header = STORAGE.lock().unwrap().get_header(0).unwrap();
    assert_eq!(
        header,
        bitcoin::blockdata::constants::genesis_block(bitcoin::network::constants::Network::Testnet)
            .header
    );
}
