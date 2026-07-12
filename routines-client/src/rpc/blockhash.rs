use core::fmt;
use std::str::FromStr;

use serde::Deserialize;

pub const BYTE_32_ARRAY_LEN: usize = 32;
pub type Byte32Array = [u8; BYTE_32_ARRAY_LEN];

#[derive(Deserialize, Default, Clone, Copy)]
pub struct BlockHash {
    #[serde(deserialize_with = "deserialize_blockhash")]
    pub blockhash: Byte32Array,
    pub last_valid_block_height: Option<u64>,
}

fn deserialize_blockhash<'de, D>(deserializer: D) -> Result<Byte32Array, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    let bytes: Byte32Array = bs58::decode(&value)
        .into_vec()
        .map_err(|error| serde::de::Error::custom(error.to_string()))?
        .try_into()
        .or(Err(serde::de::Error::custom(
            "Decoded bytes are not 32 bytes in len",
        )))?;

    Ok(bytes)
}

impl From<Byte32Array> for BlockHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self {
            blockhash: bytes,
            last_valid_block_height: Option::default(),
        }
    }
}

impl AsRef<[u8]> for BlockHash {
    fn as_ref(&self) -> &[u8] {
        &self.blockhash[..]
    }
}

impl FromStr for BlockHash {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = [0u8; BYTE_32_ARRAY_LEN];

        bs58::decode(s).onto(bytes)?;

        Ok(Self {
            blockhash: bytes,
            last_valid_block_height: Option::default(),
        })
    }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X?}", self.as_ref())
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.as_ref()).into_string())
    }
}
