# bitcoin-blockparser

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dspicher/bitcoin-blockparser/rust.yml?branch=master&logo=github" height="20">](https://github.com/dspicher/bitcoin-blockparser/actions)
[<img alt="build status" src="https://img.shields.io/codecov/c/gh/dspicher/bitcoin-blockparser?logo=codecov" height="20">](https://codecov.io/gh/dspicher/bitcoin-blockparser)
[![dependency status](https://deps.rs/repo/github/dspicher/bitcoin-blockparser/status.svg)](https://deps.rs/repo/github/dspicher/bitcoin-blockparser)

bitcoin-blockparser is a Bitcoin Blockchain Parser written in **Rust language**.

It allows extraction of various data types (blocks, transactions, scripts, public keys/hashes, balances, ...)
and UTXO dumps from Bitcoin based blockchains.

This is a fork of the [rusty-blockparser](https://github.com/gcarq/rusty-blockparser) crate, which also contains support for other cryptocurrencies.

##### **Currently Supported Blockchains:**

 `Bitcoin` and `BitcoinTestnet3`.

**IMPORANT:** It assumes a local unpruned copy of the blockchain with intact block index and blk files,
downloaded with [Bitcoin Core](https://github.com/bitcoin/bitcoin) 0.15.1+ or similar clients.
If you are not sure whether your local copy is valid you can apply `--verify` to validate the chain and block merkle trees.
If something doesn't match the parser exits.


## Usage
```
Usage: bitcoin-blockparser [OPTIONS]

Options:
      --verify
          Verifies merkle roots and block hashes
  -v...
          Increases verbosity level. Info=0, Debug=1, Trace=2 (default: 0)
  -c, --coin <NAME>
          Specify blockchain coin (default: bitcoin) [possible values: bitcoin, testnet3]
  -d, --blockchain-dir <blockchain-dir>
          Sets blockchain directory which contains blk.dat files (default: ~/.bitcoin/blocks)
  -s, --start <HEIGHT>
          Specify starting block for parsing (inclusive)
  -e, --end <HEIGHT>
          Specify last block for parsing (inclusive) (default: all known blocks)
  -h, --help
          Print help
  -V, --version
          Print version
```


## Installing

This tool should run on Windows, OS X and Linux.
All you need is `rust` and `cargo`.


### Latest Release

You can download the latest release from crates.io:
```bash
cargo install bitcoin-blockparser
```

### Build from source

```bash
git clone https://github.com/dspicher/bitcoin-blockparser.git
cd bitcoin-blockparser
cargo build --release
cargo test --release
./target/release/bitcoin-blockparser --help
```

It is important to build with `--release`, otherwise you will get a horrible performance!

*Tested on Gentoo Linux with rust-stable 1.44.1*


## Supported Transaction Types

Bitcoin and Bitcoin Testnet transactions are parsed using [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin),
this includes transactions of type P2SH, P2PKH, P2PK, P2WSH, P2WPKH, P2TR, OP_RETURN and SegWit.

## Contributing

Use the issue tracker to report problems, suggestions and questions. You may also contribute by submitting pull requests.

## TODO

* Implement Pay2MultiSig script evaluation
