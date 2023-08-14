use bitcoin_blockparser::parser::chain::ChainStorage;
use bitcoin_blockparser::parser::BlockchainParser;
use tracing_subscriber::prelude::*;

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let options =
        match bitcoin_blockparser::parse_args(&bitcoin_blockparser::command().get_matches()) {
            Ok(o) => o,
            Err(desc) => {
                tracing::error!(target: "main", "{}", desc);
                std::process::exit(1);
            }
        };

    // Apply log filter based on verbosity
    tracing::info!(target: "main", "Starting bitcoin-blockparser v{} ...", env!("CARGO_PKG_VERSION"));
    if options.verify {
        tracing::info!(target: "main", "Configured to verify merkle roots and block hashes");
    }

    let chain_storage = match ChainStorage::new(&options) {
        Ok(storage) => storage,
        Err(e) => {
            tracing::error!(
                target: "main",
                "Cannot load blockchain data from: '{}'. {}",
                options.blockchain_dir.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let mut parser = BlockchainParser::new(&options, chain_storage);
    parser.start();
    tracing::info!(target: "main", "Fin.");
}
