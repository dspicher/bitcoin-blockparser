fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn storage(datadir: &str, max_height: u64) -> bitcoin_blockparser::parser::chain::ChainStorage {
    let tempdir = tempfile::tempdir().unwrap();
    copy_dir_all(format!("tests/testdata/{datadir}"), &tempdir).unwrap();
    let options = bitcoin_blockparser::ParserOptions {
        coin: datadir.parse().unwrap(),
        verify: true,
        blockchain_dir: tempdir.into_path(),
        range: bitcoin_blockparser::BlockHeightRange::new(0, Some(max_height)).unwrap(),
    };
    bitcoin_blockparser::parser::chain::ChainStorage::new(&options).unwrap()
}
