use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// A digital radar data block's identifier.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, FromBytes, Immutable, KnownLayout)]
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
}
