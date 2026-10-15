use super::DataBlockId;

/// A wrapper that pairs a data block with its identifier.
///
/// This generic wrapper preserves the `DataBlockId` that precedes each data block
/// in the binary format, allowing consumers to access both the block type identifier
/// and the block data itself. Use [`inner()`](Self::inner) or
/// [`into_inner()`](Self::into_inner) to reach the block data.
#[derive(Debug, Clone, PartialEq)]
pub struct DataBlock<'a, T> {
    /// The data block identifier that precedes this block in the binary data.
    id: DataBlockId<'a>,

    /// The offset of the block identifier in the decoder's input, for error reporting.
    offset: usize,

    /// The inner data block.
    inner: T,
}

impl<'a, T> DataBlock<'a, T> {
    /// Creates a new data block wrapper with the given ID and inner data.
    pub(crate) fn new(id: DataBlockId<'a>, offset: usize, inner: T) -> Self {
        Self { id, offset, inner }
    }

    /// The offset of the block identifier in the decoder's input.
    #[cfg(feature = "model")]
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    /// Convert this data block to an owned version with `'static` lifetime,
    /// using the provided function to convert the inner value.
    pub fn into_owned_with<U>(self, f: impl FnOnce(T) -> U) -> DataBlock<'static, U> {
        DataBlock {
            id: self.id.into_owned(),
            offset: self.offset,
            inner: f(self.inner),
        }
    }

    /// The data block identifier that precedes this block in the binary data.
    pub fn id(&self) -> &DataBlockId<'a> {
        &self.id
    }

    /// The inner data block.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Consume this wrapper and return the inner data block.
    pub fn into_inner(self) -> T {
        self.inner
    }
}
