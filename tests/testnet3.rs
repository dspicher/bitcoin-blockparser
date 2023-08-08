mod common;

static STORAGE: once_cell::sync::Lazy<
    std::sync::Mutex<bitcoin_blockparser::parser::chain::ChainStorage>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(common::storage("testnet3", 120)));

#[test]
fn test_blockdata_parsing() {
    assert_eq!(
        STORAGE
            .lock()
            .unwrap()
            .get_block(0)
            .unwrap()
            .block_hash()
            .to_string(),
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943"
    );
    for height in 0..=120 {
        let block = STORAGE.lock().unwrap().get_block(height).unwrap();
        assert_eq!(block.txdata.len(), 1);
    }
}
