use super::raw;
use std::borrow::Cow;

/// A digital radar data block's identifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataBlockId<'a> {
    inner: Cow<'a, raw::DataBlockId>,
}

impl<'a> DataBlockId<'a> {
    /// Create a new DataBlockId wrapper from a raw DataBlockId reference.
    pub(crate) fn new(inner: &'a raw::DataBlockId) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this data block ID to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> DataBlockId<'static> {
        DataBlockId {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Data block type (raw byte), e.g. 'R'.
    pub fn data_block_type_raw(&self) -> u8 {
        self.inner.data_block_type
    }

    /// Data block name (raw bytes), e.g. "VOL".
    pub fn data_name_raw(&self) -> &[u8; 3] {
        &self.inner.data_name
    }

    /// Data block type, e.g. "R".
    pub fn data_block_type(&self) -> char {
        self.inner.data_block_type as char
    }

    /// Data block name, e.g. "VOL".
    ///
    /// The name is always 3 bytes of ASCII from the binary format. If the bytes are
    /// not valid UTF-8 (e.g. from a malformed file), invalid sequences are replaced
    /// with the Unicode replacement character.
    pub fn data_block_name(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.inner.data_name)
    }
}
