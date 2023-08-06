use clap::{Arg, Command};
use tracing_subscriber::prelude::*;

use crate::callbacks::balances::Balances;
use crate::callbacks::opreturn::OpReturn;
use crate::callbacks::simplestats::SimpleStats;
use crate::callbacks::unspentcsvdump::UnspentCsvDump;
use crate::callbacks::Callback;
use crate::parser::chain::ChainStorage;
use crate::parser::types::{Bitcoin, CoinType};
use crate::parser::BlockchainParser;

pub mod callbacks;
pub mod parser;

#[derive(Copy, Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct BlockHeightRange {
    start: u64,
    end: Option<u64>,
}

impl BlockHeightRange {
    pub fn new(start: u64, end: Option<u64>) -> anyhow::Result<Self> {
        if end.is_some() && start > end.unwrap() {
            anyhow::bail!("--start value must be lower than --end value",);
        }
        Ok(Self { start, end })
    }

    #[must_use]
    pub fn is_default(&self) -> bool {
        self.start == 0 && self.end.is_none()
    }
}

impl std::fmt::Display for BlockHeightRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let end = match self.end {
            Some(e) => e.to_string(),
            None => String::from("HEAD"),
        };
        write!(f, "{}..{}", self.start, end)
    }
}

/// Holds all available user arguments
pub struct ParserOptions {
    // Name of the callback which gets executed for each block. (See callbacks/mod.rs)
    callback: Box<dyn Callback>,
    // Holds the relevant coin parameters we need for parsing
    coin: CoinType,
    // Enable this if you want to check the chain index integrity and merkle root for each block.
    verify: bool,
    // Path to directory where blk.dat files are stored
    blockchain_dir: std::path::PathBuf,
    // Range which is considered for parsing
    range: BlockHeightRange,
}

fn command() -> Command {
    let coins = ["bitcoin", "testnet3"];
    Command::new("bitcoin-blockparser")
    .version(clap::crate_version!())
    // Add flags
    .arg(Arg::new("verify")
        .long("verify")
        .action(clap::ArgAction::SetTrue)
        .value_parser(clap::value_parser!(bool))
        .help("Verifies merkle roots and block hashes"))
    .arg(Arg::new("verbosity")
        .short('v')
        .action(clap::ArgAction::Count)
        .help("Increases verbosity level. Info=0, Debug=1, Trace=2 (default: 0)"))
    // Add options
    .arg(Arg::new("coin")
        .short('c')
        .long("coin")
        .value_name("NAME")
        .value_parser(clap::builder::PossibleValuesParser::new(coins))
        .help("Specify blockchain coin (default: bitcoin)"))
    .arg(Arg::new("blockchain-dir")
        .short('d')
        .long("blockchain-dir")
        .help("Sets blockchain directory which contains blk.dat files (default: ~/.bitcoin/blocks)"))
    .arg(Arg::new("start")
        .short('s')
        .long("start")
        .value_name("HEIGHT")
        .value_parser(clap::value_parser!(u64))
        .help("Specify starting block for parsing (inclusive)"))
    .arg(Arg::new("end")
        .short('e')
        .long("end")
        .value_name("HEIGHT")
        .value_parser(clap::value_parser!(u64))
        .help("Specify last block for parsing (inclusive) (default: all known blocks)"))
    // Add callbacks
    .subcommand(UnspentCsvDump::build_subcommand())
    .subcommand(SimpleStats::build_subcommand())
    .subcommand(Balances::build_subcommand())
    .subcommand(OpReturn::build_subcommand())
}

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let options = match parse_args(&command().get_matches()) {
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

    let mut parser = BlockchainParser::new(options, chain_storage);
    match parser.start() {
        Ok(_) => tracing::info!(target: "main", "Fin."),
        Err(why) => {
            tracing::error!("{}", why);
            std::process::exit(1);
        }
    }
}

/// Returns default directory. TODO: test on windows
fn get_absolute_blockchain_dir(coin: &CoinType) -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Unable to get home path from env!")
        .join(&coin.default_folder)
}

/// Parses args or panics if some requirements are not met.
fn parse_args(matches: &clap::ArgMatches) -> anyhow::Result<ParserOptions> {
    let verify = matches.get_flag("verify");

    let coin = matches
        .get_one::<String>("coin")
        .map_or_else(|| CoinType::from(Bitcoin), |v| v.parse().unwrap());
    let blockchain_dir = match matches.get_one::<String>("blockchain-dir") {
        Some(p) => std::path::PathBuf::from(p),
        None => get_absolute_blockchain_dir(&coin),
    };
    let start = matches.get_one::<u64>("start").copied().unwrap_or(0);
    let end = matches.get_one::<u64>("end").copied();
    let range = BlockHeightRange::new(start, end)?;

    // Set callback
    let callback: Box<dyn Callback>;
    if let Some(matches) = matches.subcommand_matches("simplestats") {
        callback = Box::new(SimpleStats::new(matches)?);
    } else if let Some(matches) = matches.subcommand_matches("unspentcsvdump") {
        callback = Box::new(UnspentCsvDump::new(matches)?);
    } else if let Some(matches) = matches.subcommand_matches("balances") {
        callback = Box::new(Balances::new(matches)?);
    } else if let Some(matches) = matches.subcommand_matches("opreturn") {
        callback = Box::new(OpReturn::new(matches)?);
    } else {
        clap::error::Error::<clap::error::DefaultFormatter>::raw(
            clap::error::ErrorKind::MissingSubcommand,
            "error: No valid callback specified.\nFor more information try --help",
        )
        .exit();
    }

    let options = ParserOptions {
        callback,
        coin,
        verify,
        blockchain_dir,
        range,
    };
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_subcommand() {
        let tmp_dir = tempfile::tempdir().unwrap();
        parse_args(&command().get_matches_from([
            "bitcoin-blockparser",
            "unspentcsvdump",
            tmp_dir.path().to_str().unwrap(),
        ]))
        .unwrap();
        parse_args(&command().get_matches_from(["bitcoin-blockparser", "simplestats"])).unwrap();
        parse_args(&command().get_matches_from([
            "bitcoin-blockparser",
            "balances",
            tmp_dir.path().to_str().unwrap(),
        ]))
        .unwrap();
        parse_args(&command().get_matches_from(["bitcoin-blockparser", "opreturn"])).unwrap();
    }

    #[test]
    fn test_args_coin() {
        let args = ["bitcoin-blockparser", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.coin.name, "Bitcoin");

        let args = ["bitcoin-blockparser", "-c", "testnet3", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.coin.name, "TestNet3");
    }

    #[test]
    fn test_args_verify() {
        let args = ["bitcoin-blockparser", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert!(!options.verify);

        let args = ["bitcoin-blockparser", "--verify", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert!(options.verify);
    }

    #[test]
    fn test_args_blockchain_dir() {
        let args = ["bitcoin-blockparser", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.blockchain_dir,
            dirs::home_dir()
                .unwrap()
                .join(std::path::Path::new(".bitcoin").join("blocks"))
        );

        let args = ["bitcoin-blockparser", "-d", "foo", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.blockchain_dir.to_str().unwrap(), "foo");

        let args = [
            "bitcoin-blockparser",
            "--blockchain-dir",
            "foo",
            "simplestats",
        ];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.blockchain_dir.to_str().unwrap(), "foo");
    }

    #[test]
    fn test_args_start() {
        let args = ["bitcoin-blockparser", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 0,
                end: None
            }
        );

        let args = ["bitcoin-blockparser", "-s", "10", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 10,
                end: None
            }
        );

        let args = ["bitcoin-blockparser", "--start", "10", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 10,
                end: None
            }
        );
    }

    #[test]
    fn test_args_end() {
        let args = ["bitcoin-blockparser", "-e", "10", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 0,
                end: Some(10)
            }
        );

        let args = ["bitcoin-blockparser", "--end", "10", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 0,
                end: Some(10)
            }
        );
    }

    #[test]
    fn test_args_start_and_end() {
        let args = ["bitcoin-blockparser", "-s", "1", "-e", "2", "simplestats"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 1,
                end: Some(2)
            }
        );

        let args = ["bitcoin-blockparser", "-s", "2", "-e", "1", "simplestats"];
        assert!(parse_args(&command().get_matches_from(args)).is_err());
    }
}
