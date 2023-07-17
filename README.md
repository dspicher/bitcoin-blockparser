# rusty-blockparser

rusty-blockparser is a Bitcoin Blockchain Parser written in **Rust language**.

It allows extraction of various data types (blocks, transactions, scripts, public keys/hashes, balances, ...)
and UTXO dumps from Bitcoin based blockchains.

##### **Currently Supported Blockchains:**

 `Bitcoin` and `BitcoinTestnet3`.

**IMPORANT:** It assumes a local unpruned copy of the blockchain with intact block index and blk files,
downloaded with [Bitcoin Core](https://github.com/bitcoin/bitcoin) 0.15.1+ or similar clients.
If you are not sure whether your local copy is valid you can apply `--verify` to validate the chain and block merkle trees.
If something doesn't match the parser exits.


## Usage
```
Usage: rusty-blockparser [OPTIONS] [COMMAND]

Commands:
  unspentcsvdump  Dumps the unspent outputs to CSV file
  csvdump         Dumps the whole blockchain into CSV files
  simplestats     Shows various Blockchain stats
  balances        Dumps all addresses with non-zero balance to CSV file
  opreturn        Shows embedded OP_RETURN data that is representable as UTF8
  help            Print this message or the help of the given subcommand(s)

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
### Example

To make a `unspentcsvdump` of the Bitcoin blockchain your command would look like this:
```
# ./blockparser unspentcsvdump /path/to/dump/
[6:02:53] INFO - main: Starting rusty-blockparser v0.7.0 ...
[6:02:53] INFO - index: Reading index from ~/.bitcoin/blocks/index ...
[6:02:54] INFO - index: Got longest chain with 639626 blocks ...
[6:02:54] INFO - blkfile: Reading files from ~/.bitcoin/blocks ...
[6:02:54] INFO - parser: Parsing Bitcoin blockchain (range=0..) ...
[6:02:54] INFO - callback: Using `unspentcsvdump` with dump folder: /path/to/dump ...
[6:03:04] INFO - parser: Status: 130885 Blocks processed. (left: 508741, avg: 13088 blocks/sec)
...
[10:28:47] INFO - parser: Status: 639163 Blocks processed. (left:    463, avg:    40 blocks/sec)
[10:28:57] INFO - parser: Status: 639311 Blocks processed. (left:    315, avg:    40 blocks/sec)
[10:29:07] INFO - parser: Status: 639452 Blocks processed. (left:    174, avg:    40 blocks/sec)
[10:29:17] INFO - parser: Status: 639596 Blocks processed. (left:     30, avg:    40 blocks/sec)
[10:29:19] INFO - parser: Done. Processed 639626 blocks in 266.43 minutes. (avg:    40 blocks/sec)
[10:32:01] INFO - callback: Done.
Dumped all 639626 blocks:
        -> transactions: 549390991
        -> inputs:       1347165535
        -> outputs:      1359449320
[10:32:01] INFO - main: Fin.
```


## Installing

This tool should run on Windows, OS X and Linux.
All you need is `rust` and `cargo`.


### Latest Release

You can download the latest release from crates.io:
```bash
cargo install rusty-blockparser
```

### Build from source

```bash
git clone https://github.com/gcarq/rusty-blockparser.git
cd rusty-blockparser
cargo build --release
cargo test --release
./target/release/rusty-blockparser --help
```

It is important to build with `--release`, otherwise you will get a horrible performance!

*Tested on Gentoo Linux with rust-stable 1.44.1*


## Supported Transaction Types

Bitcoin and Bitcoin Testnet transactions are parsed using [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin),
this includes transactions of type P2SH, P2PKH, P2PK, P2WSH, P2WPKH, P2TR, OP_RETURN and SegWit.


## Memory Usage
The required memory usage depends on the used callback:

* simplestats: ~100MB
* csvdump: ~100M
* unspentcsvdump: ~18GB
* balances: ~18GB

NOTE: Those values are taken from parsing to block height 639631 (17.07.2020).

## Callbacks

Callbacks are built on top of the core parser. They can be implemented to extract specific types of information.

* `balances`: dumps all addresses with a non-zero balance.
    The csv file is in the following format:
    ```
    balances.csv
    address ; balance
    ```

* `unspentcsvdump`: dumps all UTXOs along with the address balance.
    The csv file is in the following format:
    ```
    unspent.csv
    txid ; indexOut ; height ; value ; address
    ```
    NOTE: The total size of the csv dump is at least 8 GiB (height 635000).

* `opreturn`: shows transactions with embedded OP_RETURN data that is representable as UTF8.

* `csvdump`: dumps all parsed data as CSV files into the specified `folder`. See [Usage](#Usage) for an example. I chose CSV dumps instead of  an active db-connection because `LOAD DATA INFILE` is the most performant way for bulk inserts.
    The files are in the following format:
    ```
    blocks.csv
    block_hash ; height ; version ; blocksize ; hashPrev ; hashMerkleRoot ; nTime ; nBits ; nNonce
    ```
    ```
    transactions.csv
    txid ; hashBlock ; version ; lockTime
    ```
    ```
    tx_in.csv
    txid ; hashPrevOut ; indexPrevOut ; scriptSig ; sequence
    ```
    ```
    tx_out.csv
    txid ; indexOut ; height ; value ; scriptPubKey ; address
    ```
    If unclear what some of these fields are, see the [block](https://en.bitcoin.it/wiki/Protocol_documentation#block) and [transaction](https://en.bitcoin.it/wiki/Protocol_documentation#tx) specifications.


* `simplestats`: prints some blockchain statistics like block count, transaction count, avg transactions per block, largest transaction, transaction types etc.

You can also define custom callbacks. A callback gets called at startup, on each block and at the end. See [src/callbacks/mod.rs](src/callbacks/mod.rs) for more information.


## Contributing

Use the issue tracker to report problems, suggestions and questions. You may also contribute by submitting pull requests.

If you find this project helpful, please consider making a donation:
`1LFidBTeg5joAqjw35ksebiNkVM8azFM1K`

## TODO

* Implement Pay2MultiSig script evaluation
