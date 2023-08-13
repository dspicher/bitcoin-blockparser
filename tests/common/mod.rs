pub fn storage(datadir: &str, max_height: u64) -> bitcoin_blockparser::parser::chain::ChainStorage {
    let options = bitcoin_blockparser::ParserOptions {
        callback: Box::<bitcoin_blockparser::callbacks::simplestats::SimpleStats>::default(),
        coin: datadir.parse().unwrap(),
        verify: true,
        blockchain_dir: std::path::PathBuf::from(format!("tests/testdata/{datadir}")),
        range: bitcoin_blockparser::BlockHeightRange::new(0, Some(max_height)).unwrap(),
    };
    let storage = bitcoin_blockparser::parser::chain::ChainStorage::new(&options).unwrap();

    // Discard transient diff on LevelDB files
    std::process::Command::new("git")
        .args(["checkout", format!("tests/testdata/{datadir}").as_str()])
        .output()
        .unwrap();
    storage
}
