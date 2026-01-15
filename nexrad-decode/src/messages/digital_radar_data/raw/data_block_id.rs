use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// A digital radar data block's identifier.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, FromBytes, Immutable, KnownLayout)]
pub struct DataBlockId {
    /// Data block type, e.g. "R".
    pub(crate) data_block_type: u8,

    /// Data block name, e.g. "VOL".
    pub(crate) data_name: [u8; 3],
}
