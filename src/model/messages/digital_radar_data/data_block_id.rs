use serde::Deserialize;
use std::fmt::Debug;

/// A digital radar data block's identifier.
#[derive(Deserialize)]
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

impl Debug for DataBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataBlockId")
            .field("data_block_type", &self.data_block_type())
            .field("data_block_name", &self.data_block_name())
            .finish()
    }
}
