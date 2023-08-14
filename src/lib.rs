use clap::{Arg, Command};

use crate::parser::types::{Bitcoin, CoinType};

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

pub struct ParserOptions {
    pub coin: CoinType,
    pub verify: bool,
    pub blockchain_dir: std::path::PathBuf,
    pub range: BlockHeightRange,
}

#[must_use]
pub fn command() -> Command {
    let coins = ["bitcoin", "testnet3"];
    Command::new("bitcoin-blockparser")
    .version(clap::crate_version!())
    .arg(Arg::new("verify")
        .long("verify")
        .action(clap::ArgAction::SetTrue)
        .value_parser(clap::value_parser!(bool))
        .help("Verifies merkle roots and block hashes"))
    .arg(Arg::new("verbosity")
        .short('v')
        .action(clap::ArgAction::Count)
        .help("Increases verbosity level. Info=0, Debug=1, Trace=2 (default: 0)"))
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
}

fn get_absolute_blockchain_dir(coin: &CoinType) -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Unable to get home path from env!")
        .join(&coin.default_folder)
}

pub fn parse_args(matches: &clap::ArgMatches) -> anyhow::Result<ParserOptions> {
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

    let options = ParserOptions {
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
    fn test_args_coin() {
        let args = ["bitcoin-blockparser"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.coin.name, "Bitcoin");

        let args = ["bitcoin-blockparser", "-c", "testnet3"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.coin.name, "TestNet3");
    }

    #[test]
    fn test_args_verify() {
        let args = ["bitcoin-blockparser"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert!(!options.verify);

        let args = ["bitcoin-blockparser", "--verify"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert!(options.verify);
    }

    #[test]
    fn test_args_blockchain_dir() {
        let args = ["bitcoin-blockparser"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.blockchain_dir,
            dirs::home_dir()
                .unwrap()
                .join(std::path::Path::new(".bitcoin").join("blocks"))
        );

        let args = ["bitcoin-blockparser", "-d", "foo"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.blockchain_dir.to_str().unwrap(), "foo");

        let args = ["bitcoin-blockparser", "--blockchain-dir", "foo"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(options.blockchain_dir.to_str().unwrap(), "foo");
    }

    #[test]
    fn test_args_start() {
        let args = ["bitcoin-blockparser"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 0,
                end: None
            }
        );

        let args = ["bitcoin-blockparser", "-s", "10"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 10,
                end: None
            }
        );

        let args = ["bitcoin-blockparser", "--start", "10"];
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
        let args = ["bitcoin-blockparser", "-e", "10"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 0,
                end: Some(10)
            }
        );

        let args = ["bitcoin-blockparser", "--end", "10"];
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
        let args = ["bitcoin-blockparser", "-s", "1", "-e", "2"];
        let options = parse_args(&command().get_matches_from(args)).unwrap();
        assert_eq!(
            options.range,
            BlockHeightRange {
                start: 1,
                end: Some(2)
            }
        );

        let args = ["bitcoin-blockparser", "-s", "2", "-e", "1"];
        assert!(parse_args(&command().get_matches_from(args)).is_err());
    }
}
