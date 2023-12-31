use bitcoin::hashes::sha256d;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub trait Coin {
    fn name(&self) -> String;

    fn magic(&self) -> u32;

    fn version_id(&self) -> u8;

    fn genesis(&self) -> sha256d::Hash;

    fn default_folder(&self) -> PathBuf;
}

pub struct Bitcoin;
pub struct TestNet3;

impl Coin for Bitcoin {
    fn name(&self) -> String {
        String::from("Bitcoin")
    }
    fn magic(&self) -> u32 {
        0xd9b4_bef9
    }
    fn version_id(&self) -> u8 {
        0x00
    }
    fn genesis(&self) -> sha256d::Hash {
        sha256d::Hash::from_str("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f")
            .unwrap()
    }
    fn default_folder(&self) -> PathBuf {
        Path::new(".bitcoin").join("blocks")
    }
}

impl Coin for TestNet3 {
    fn name(&self) -> String {
        String::from("TestNet3")
    }
    fn magic(&self) -> u32 {
        0x0709_110b
    }
    fn version_id(&self) -> u8 {
        0x6f
    }
    fn genesis(&self) -> sha256d::Hash {
        sha256d::Hash::from_str("000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943")
            .unwrap()
    }
    fn default_folder(&self) -> PathBuf {
        Path::new(".bitcoin").join("testnet3")
    }
}

#[derive(Clone)]
pub struct CoinType {
    pub name: String,
    pub magic: u32,
    pub version_id: u8,
    pub genesis_hash: sha256d::Hash,
    pub default_folder: PathBuf,
}

impl Default for CoinType {
    fn default() -> Self {
        CoinType::from(Bitcoin)
    }
}

impl<T: Coin> From<T> for CoinType {
    fn from(coin: T) -> Self {
        CoinType {
            name: coin.name(),
            magic: coin.magic(),
            version_id: coin.version_id(),
            genesis_hash: coin.genesis(),
            default_folder: coin.default_folder(),
        }
    }
}

impl FromStr for CoinType {
    type Err = anyhow::Error;
    fn from_str(coin_name: &str) -> anyhow::Result<Self> {
        match coin_name {
            "bitcoin" => Ok(CoinType::from(Bitcoin)),
            "testnet3" => Ok(CoinType::from(TestNet3)),
            n => {
                anyhow::bail!("There is no impl for `{}`!", n);
            }
        }
    }
}
