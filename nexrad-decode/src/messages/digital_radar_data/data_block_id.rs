use std::fmt::Debug;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

/// A digital radar data block's identifier.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, TryFromBytes, Immutable, KnownLayout)]
pub struct DataBlockId {
    /// Data block type, e.g. "R".
    pub data_block_type: u8,

    /// Data block name, e.g. "VOL".
    pub data_name: [u8; 3],
}

impl DataBlockId {
    /// Data block type, e.g. "R".
    pub fn data_block_type(&self) -> char {
        self.data_block_type as char
    }

    /// Data block name, e.g. "VOL".
    pub fn data_block_name(&self) -> String {
        String::from_utf8_lossy(&self.data_name).to_string()
    }

    /// Decodes a reference to a DataBlockId from a byte slice, returning the ID and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a DataBlockId from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (id, _) = Self::decode_ref(bytes)?;
        Ok(id.clone())
    }
}
