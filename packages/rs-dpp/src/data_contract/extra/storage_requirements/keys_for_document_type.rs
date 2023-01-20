use anyhow::bail;
use serde_repr::*;
use std::convert::TryFrom;
/// The Storage Key requirements
#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Copy, Clone)]
pub enum StorageKeyRequirements {
    Unique = 0,
    UniqueReplaceable = 1,
    Multiple = 2,
    MultipleWithMain = 3,
    MultipleIndexed = 4,
}

impl TryFrom<u8> for StorageKeyRequirements {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unique),
            1 => Ok(Self::UniqueReplaceable),
            2 => Ok(Self::Multiple),
            3 => Ok(Self::MultipleWithMain),
            4 => Ok(Self::MultipleIndexed),
            value => bail!("unrecognized storage key requirements: {}", value),
        }
    }
}
