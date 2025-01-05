use crate::messages::digital_radar_data::DataBlockId;

/// A digital radar message data block.
#[derive(Debug, Clone, PartialEq)]
pub struct DataBlock<B> {
    /// Data block identifier.
    pub id: DataBlockId,

    /// Data block contents.
    pub data: B,
}

impl<B> DataBlock<B> {
    /// Create a new data block with the given ID and data.
    pub(crate) fn new(id: DataBlockId, data: B) -> Self {
        Self { id, data }
    }
}
