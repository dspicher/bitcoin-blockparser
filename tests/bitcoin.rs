mod common;

fn storage() -> bitcoin_blockparser::parser::chain::ChainStorage {
    common::storage("bitcoin", 170)
}

fn parser() -> bitcoin_blockparser::parser::BlockchainParser {
    common::parser("bitcoin", 170)
}

#[test]
fn test_bitcoin_genesis() {
    let genesis = storage().get_block(0).unwrap();
    assert_eq!(
        genesis,
        bitcoin::blockdata::constants::genesis_block(bitcoin::network::constants::Network::Bitcoin)
    );

    assert_eq!(285, genesis.size());

    assert_eq!(0x0000_0001, genesis.header.version.to_consensus());
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000000000",
        format!("{}", &genesis.header.prev_blockhash)
    );
    assert_eq!(
        "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        format!("{}", &genesis.header.merkle_root)
    );
    assert_eq!(
        "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        format!("{}", &genesis.header.block_hash())
    );

    assert_eq!(
        &genesis.header.merkle_root,
        &genesis.compute_merkle_root().unwrap()
    );
    assert_eq!(1_231_006_505, genesis.header.time);
    assert_eq!(0x1d00_ffff, genesis.header.bits.to_consensus());
    assert_eq!(2_083_236_893, genesis.header.nonce);

    assert_eq!(0x01, genesis.txdata.len());
    assert_eq!(0x0000_0001, genesis.txdata[0].version);

    assert_eq!(0x01, genesis.txdata[0].input.len());
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000000000",
        format!("{}", &genesis.txdata[0].input[0].previous_output.txid)
    );
    assert_eq!(0xffff_ffff, genesis.txdata[0].input[0].previous_output.vout);
    assert_eq!(0x4d, genesis.txdata[0].input[0].script_sig.len());
    let script = "04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73";
    assert_eq!(
        hex::decode(script).unwrap(),
        genesis.txdata[0].input[0].script_sig.as_bytes()
    );
    assert!(script.contains(
        hex::encode(
            "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks".as_bytes()
        )
        .as_str()
    ));
    assert_eq!(
        0xffff_ffff,
        genesis.txdata[0].input[0].sequence.to_consensus_u32()
    );

    assert_eq!(0x01, genesis.txdata[0].output.len());
    assert_eq!(
        u64::from_be(0x00f2_052a_0100_0000),
        genesis.txdata[0].output[0].value
    );
    assert_eq!(0x43, genesis.txdata[0].output[0].script_pubkey.len());

    let script = hex::decode("4104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac").unwrap();
    assert_eq!(script, genesis.txdata[0].output[0].script_pubkey.as_bytes());
    assert_eq!(0x0000_0000, genesis.txdata[0].lock_time.to_consensus_u32());
}

#[test]
fn test_genesis_header() {
    let header = storage().get_header(0).unwrap();
    assert_eq!(
        header,
        bitcoin::blockdata::constants::genesis_block(bitcoin::network::constants::Network::Bitcoin)
            .header
    );
}

#[test]
fn test_blockdata_parsing() {
    let mut storage = storage();
    for height in 0..=169 {
        let block = storage.get_block(height).unwrap();
        assert_eq!(block.txdata.len(), 1);
    }
    let first_tx_block = storage.get_block(170).unwrap();
    assert_eq!(first_tx_block.txdata.len(), 2);

    let tx = first_tx_block.txdata.get(1).unwrap();
    let to_hal_finney = tx.output.get(0).unwrap();
    assert_eq!(to_hal_finney.value, 10 * bitcoin::Amount::ONE_BTC.to_sat());
    assert!(to_hal_finney.script_pubkey.is_p2pk());
    assert_eq!(
        tx.output.get(1).unwrap().value,
        40 * bitcoin::Amount::ONE_BTC.to_sat()
    );
}

#[test]
fn test_headers() {
    let mut storage = storage();
    for height in 0..=169 {
        storage.get_header(height).unwrap();
    }
}

#[test]
fn test_blocks_db() {
    let mut parser = parser();
    parser.start().unwrap();
    assert_eq!(parser.db().blocks_count().unwrap(), 171);
}
