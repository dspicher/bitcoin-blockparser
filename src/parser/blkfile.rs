use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::parser::reader::BlockchainRead;

#[derive(Debug)]
pub struct BlkFile {
    pub path: PathBuf,
    pub size: u64,
    reader: Option<std::io::BufReader<File>>,
}

impl BlkFile {
    fn new(path: PathBuf, size: u64) -> BlkFile {
        BlkFile {
            path,
            size,
            reader: None,
        }
    }

    fn open(&mut self) -> anyhow::Result<&mut std::io::BufReader<File>> {
        if self.reader.is_none() {
            tracing::debug!(target: "blkfile", "Opening {} ...", &self.path.display());
            self.reader = Some(std::io::BufReader::new(File::open(&self.path)?));
        }
        Ok(self.reader.as_mut().unwrap())
    }

    pub fn close(&mut self) {
        tracing::debug!(target: "blkfile", "Closing {} ...", &self.path.display());
        if self.reader.is_some() {
            self.reader = None;
        }
    }

    pub fn read_header(
        &mut self,
        offset: u64,
    ) -> anyhow::Result<bitcoin::blockdata::block::Header> {
        let reader = self.open()?;
        reader.seek(SeekFrom::Start(offset))?;
        reader.read_header()
    }

    pub fn read_block(&mut self, offset: u64) -> anyhow::Result<bitcoin::Block> {
        let reader = self.open()?;
        reader.seek(SeekFrom::Start(offset))?;
        reader.read_block()
    }

    pub fn from_path(path: &Path) -> anyhow::Result<HashMap<u64, BlkFile>> {
        tracing::info!(target: "blkfile", "Reading files from {} ...", path.display());
        let mut collected = HashMap::with_capacity(4000);

        for entry in std::fs::read_dir(path)? {
            match entry {
                Ok(de) => {
                    let path = BlkFile::resolve_path(&de)?;
                    if !path.is_file() {
                        continue;
                    }

                    let file_name = String::from(
                        path.as_path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .context("invalid path")?,
                    );
                    if let Some(index) = BlkFile::parse_blk_index(&file_name, "blk", ".dat") {
                        let size = std::fs::metadata(path.as_path())?.len();
                        tracing::trace!(target: "blkfile", "Adding {} ... (index: {}, size: {})", path.display(), index, size);
                        collected.insert(index, BlkFile::new(path, size));
                    }
                }
                Err(msg) => {
                    tracing::warn!(target: "blkfile", "Unable to read blk file!: {}", msg);
                }
            }
        }

        tracing::trace!(target: "blkfile", "Found {} blk files", collected.len());
        if collected.is_empty() {
            Err(anyhow::anyhow!("No blk files found!"))
        } else {
            Ok(collected)
        }
    }

    fn resolve_path(entry: &DirEntry) -> std::io::Result<PathBuf> {
        if entry.file_type()?.is_symlink() {
            std::fs::read_link(entry.path())
        } else {
            Ok(entry.path())
        }
    }

    fn parse_blk_index(file_name: &str, prefix: &str, ext: &str) -> Option<u64> {
        if file_name.starts_with(prefix) && file_name.ends_with(ext) {
            file_name[prefix.len()..(file_name.len() - ext.len())]
                .parse::<u64>()
                .ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blk_index() {
        let prefix = "blk";
        let ext = ".dat";

        assert_eq!(
            0,
            BlkFile::parse_blk_index("blk00000.dat", prefix, ext).unwrap()
        );
        assert_eq!(
            6,
            BlkFile::parse_blk_index("blk6.dat", prefix, ext).unwrap()
        );
        assert_eq!(
            1202,
            BlkFile::parse_blk_index("blk1202.dat", prefix, ext).unwrap()
        );
        assert_eq!(
            13_412_451,
            BlkFile::parse_blk_index("blk13412451.dat", prefix, ext).unwrap()
        );
        assert!(BlkFile::parse_blk_index("blkindex.dat", prefix, ext).is_none());
        assert!(BlkFile::parse_blk_index("invalid.dat", prefix, ext).is_none());
    }
}
