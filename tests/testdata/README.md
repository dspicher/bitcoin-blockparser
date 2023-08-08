Bitcoin Chain Prefixes
======================

Contains Bitcoin block data as dumped by Bitcoin Core after syncing up to a small height.

This data can be generated roughly as follows:
 1. Start bitcoind on the desired network
 1. Sync a small number of initial blocks, then kill the daemon
 1. Copy the `blocks` content over to the `testdata` directory
 1. Prune LevelDB:
    1. Set the correct API endpoint, max block height and path to the local DB in `clean_leveldb.py`
    1. Run `python clean_leveldb.py` which will create a pruned DB, removing all superfluous data
 1. Prune trailing zero bytes from `.dat` file
    1. `sed '$ s/\x00*$//' blk00000.dat > blk00000.dat.stripped`

## Mainnet

Synced up to height 200.
