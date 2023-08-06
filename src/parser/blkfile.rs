use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::parser::reader::BlockchainRead;

/// Holds all necessary data about a raw blk file
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

    /// Opens the file handle (does nothing if the file has been opened already)
    fn open(&mut self) -> anyhow::Result<&mut std::io::BufReader<File>> {
        if self.reader.is_none() {
            log::debug!(target: "blkfile", "Opening {} ...", &self.path.display());
            self.reader = Some(std::io::BufReader::new(File::open(&self.path)?));
        }
        Ok(self.reader.as_mut().unwrap())
    }

    /// Closes the file handle
    pub fn close(&mut self) {
        log::debug!(target: "blkfile", "Closing {} ...", &self.path.display());
        if self.reader.is_some() {
            self.reader = None;
        }
    }

    pub fn read_block(&mut self, offset: u64) -> anyhow::Result<bitcoin::Block> {
        let reader = self.open()?;
        reader.seek(SeekFrom::Start(offset))?;
        reader.read_block()
    }

    /// Collects all blk*.dat paths in the given directory
    pub fn from_path(path: &Path) -> anyhow::Result<HashMap<u64, BlkFile>> {
        log::info!(target: "blkfile", "Reading files from {} ...", path.display());
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
                    // Check if it's a valid blk file
                    if let Some(index) = BlkFile::parse_blk_index(&file_name, "blk", ".dat") {
                        // Build BlkFile structures
                        let size = std::fs::metadata(path.as_path())?.len();
                        log::trace!(target: "blkfile", "Adding {} ... (index: {}, size: {})", path.display(), index, size);
                        collected.insert(index, BlkFile::new(path, size));
                    }
                }
                Err(msg) => {
                    log::warn!(target: "blkfile", "Unable to read blk file!: {}", msg);
                }
            }
        }

        log::trace!(target: "blkfile", "Found {} blk files", collected.len());
        if collected.is_empty() {
            Err(anyhow::anyhow!("No blk files found!"))
        } else {
            Ok(collected)
        }
    }

    /// Resolves a PathBuf for the given entry.
    /// Also resolves symlinks if present.
    fn resolve_path(entry: &DirEntry) -> std::io::Result<PathBuf> {
        if entry.file_type()?.is_symlink() {
            std::fs::read_link(entry.path())
        } else {
            Ok(entry.path())
        }
    }

    /// Identifies blk file and parses index
    /// Returns None if this is no blk file
    fn parse_blk_index(file_name: &str, prefix: &str, ext: &str) -> Option<u64> {
        if file_name.starts_with(prefix) && file_name.ends_with(ext) {
            // Parse blk_index, this means we extract 42 from blk000042.dat
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
