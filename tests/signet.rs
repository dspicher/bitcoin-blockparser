mod common;

static STORAGE: once_cell::sync::Lazy<
    std::sync::Mutex<bitcoin_blockparser::parser::chain::ChainStorage>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(common::storage("signet", 2)));

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
        "00000008819873e925422c1ff0f99f7cc9bbb232af63a077a480a3633bee1ef6"
    );
    for height in 0..=2 {
        let block = STORAGE.lock().unwrap().get_block(height).unwrap();
        assert_eq!(block.txdata.len(), 1);
    }
}
