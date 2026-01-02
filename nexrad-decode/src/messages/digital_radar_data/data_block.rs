use crate::messages::digital_radar_data::DataBlockId;
use std::borrow::Cow;
use std::ops::Deref;

/// A wrapper that pairs a data block with its identifier.
///
/// This generic wrapper preserves the `DataBlockId` that precedes each data block
/// in the binary format, allowing consumers to access both the block type identifier
/// and the block data itself.
///
/// This type implements `Deref<Target = T>`, so you can call methods on the inner
/// type directly through the wrapper.
#[derive(Debug, Clone, PartialEq)]
pub struct DataBlock<'a, T> {
    /// The data block identifier that precedes this block in the binary data.
    pub id: Cow<'a, DataBlockId>,

    /// The inner data block.
    pub inner: T,
}

impl<'a, T> DataBlock<'a, T> {
    /// Creates a new data block wrapper with the given ID and inner data.
    pub fn new(id: Cow<'a, DataBlockId>, inner: T) -> Self {
        Self { id, inner }
    }

    /// Convert this data block to an owned version with `'static` lifetime,
    /// using the provided function to convert the inner value.
    pub fn into_owned_with<U>(self, f: impl FnOnce(T) -> U) -> DataBlock<'static, U> {
        DataBlock {
            id: Cow::Owned(self.id.into_owned()),
            inner: f(self.inner),
        }
    }
}

impl<'a, T> Deref for DataBlock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
