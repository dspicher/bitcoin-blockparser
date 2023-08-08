Bitcoin Chain Prefixes
======================

Contains Bitcoin block data as dumped by Bitcoin Core after syncing up to a small height.

This data can be generated roughly as follows:
 1. Start bitcoind on the desired network
 1. Sync a small number of initial blocks, then kill the daemon
 1. Prune LevelDB
    1. Remove non-block entries (the key doesn't begin with `b'b'`)
    1. [Copy entries over](https://github.com/wbolster/plyvel/issues/153#issuecomment-1669387180) into a new database, this results in a dramatic serialization size decrease
 1. Prune trailing zero bytes from `.dat` file
    1. `sed '$ s/\x00*$//' blk00000.dat > blk00000.dat.stripped`

## Mainnet

Synced up to height 200.
