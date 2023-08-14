use std::io::Read;

use bitcoin::hashes::{sha256d, Hash};

use rusty_leveldb::{LdbIterator, Options, DB};

use crate::ParserOptions;

const BLOCK_VALID_CHAIN: u64 = 4;
const BLOCK_HAVE_DATA: u64 = 8;

pub struct ChainIndex {
    max_height: u64,
    block_index: std::collections::HashMap<u64, BlockIndexRecord>,
    max_height_blk_index: std::collections::HashMap<u64, u64>,
}

impl ChainIndex {
    pub fn new(options: &ParserOptions) -> anyhow::Result<Self> {
        let path = options.blockchain_dir.join("index");
        let mut block_index = get_block_index(&path)?;
        let mut max_height_blk_index = std::collections::HashMap::new();

        for (height, index_record) in &block_index {
            match max_height_blk_index.get(&index_record.blk_index) {
                Some(cur_height) if height > cur_height => {
                    max_height_blk_index.insert(index_record.blk_index, *height);
                }
                None => {
                    max_height_blk_index.insert(index_record.blk_index, *height);
                }
                _ => {}
            }
        }

        let min_height = options.range.start;
        let max_known_height = *block_index.keys().max().unwrap();
        let max_height = match options.range.end {
            Some(height) if height < max_known_height => height,
            Some(_) | None => max_known_height,
        };

        if !options.range.is_default() {
            tracing::info!(target: "index", "Trimming block index from height {} to {} ...", min_height, max_height);
            block_index.retain(|height, _| {
                *height >= min_height.saturating_sub(1) && *height <= max_height
            });
        }

        Ok(Self {
            max_height,
            block_index,
            max_height_blk_index,
        })
    }

    pub fn get(&self, height: u64) -> Option<&BlockIndexRecord> {
        self.block_index.get(&height)
    }

    pub fn max_height(&self) -> u64 {
        self.max_height
    }

    pub fn max_height_by_blk(&self, blk_index: u64) -> u64 {
        *self.max_height_blk_index.get(&blk_index).unwrap()
    }
}

pub struct BlockIndexRecord {
    pub block_hash: sha256d::Hash,
    pub blk_index: u64,
    pub data_offset: u64,
    version: u64,
    height: u64,
    status: u64,
    tx_count: u64,
}

impl BlockIndexRecord {
    fn from(key: &[u8], values: &[u8]) -> anyhow::Result<Self> {
        let mut reader = std::io::Cursor::new(values);

        let block_hash: [u8; 32] = key.try_into().expect("leveldb: malformed blockhash");
        let version = read_varint(&mut reader)?;
        let height = read_varint(&mut reader)?;
        let status = read_varint(&mut reader)?;
        let tx_count = read_varint(&mut reader)?;
        let blk_index = read_varint(&mut reader)?;
        let data_offset = read_varint(&mut reader)?;

        Ok(BlockIndexRecord {
            block_hash: sha256d::Hash::from_byte_array(block_hash),
            version,
            height,
            status,
            tx_count,
            blk_index,
            data_offset,
        })
    }
}

impl std::fmt::Debug for BlockIndexRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockIndexRecord")
            .field("block_hash", &self.block_hash)
            .field("version", &self.version)
            .field("height", &self.height)
            .field("status", &self.status)
            .field("n_tx", &self.tx_count)
            .field("n_file", &self.blk_index)
            .field("n_data_pos", &self.data_offset)
            .finish()
    }
}

pub fn get_block_index(
    path: &std::path::Path,
) -> anyhow::Result<std::collections::HashMap<u64, BlockIndexRecord>> {
    tracing::info!(target: "index", "Reading index from {} ...", path.display());

    let mut block_index = std::collections::HashMap::with_capacity(1_000_000);
    let mut db_iter = DB::open(path, Options::default())?.new_iter()?;
    let (mut key, mut value) = (vec![], vec![]);

    while db_iter.advance() {
        db_iter.current(&mut key, &mut value);
        if is_block_index_record(&key) {
            let record = BlockIndexRecord::from(&key[1..], &value)?;
            if record.status & (BLOCK_VALID_CHAIN | BLOCK_HAVE_DATA) > 0 {
                block_index.insert(record.height, record);
            }
        }
    }
    tracing::info!(target: "index", "Got longest chain with {} blocks ...", block_index.len());
    Ok(block_index)
}

#[inline]
fn is_block_index_record(data: &[u8]) -> bool {
    *data.first().unwrap() == b'b'
}

/// TODO: this is a wonky 1:1 translation from https://github.com/bitcoin/bitcoin
/// It is NOT the same as CompactSize.
fn read_varint(reader: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<u64> {
    let mut n = 0;
    loop {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;
        let ch_data = buf[0];
        assert!(n <= u64::MAX >> 7, "size too large");
        n = (n << 7) | u64::from(ch_data & 0x7F);
        if ch_data & 0x80 > 0 {
            assert!(n != u64::MAX, "size too large");
            n += 1;
        } else {
            break;
        }
    }
    Ok(n)
}
