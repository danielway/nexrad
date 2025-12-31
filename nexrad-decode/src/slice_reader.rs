use crate::result::{Error, Result};
use zerocopy::FromBytes;

/// A wrapper around a byte slice that tracks the current read position.
///
/// This provides position tracking for zero-copy parsing, allowing callers to
/// know their offset in the source data for debugging and error reporting.
#[derive(Debug, Clone)]
pub struct SliceReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    /// Creates a new SliceReader starting at position 0.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Returns the current byte position in the slice.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the remaining unread bytes.
    pub fn remaining(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    /// Advances the position by `n` bytes.
    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// Returns a typed reference to the next `T` and advances the reader past it.
    pub(crate) fn take_ref<T>(&mut self) -> Result<&'a T>
    where
        T: zerocopy::FromBytes + zerocopy::KnownLayout + zerocopy::Immutable,
    {
        let remaining = self.remaining();
        let (v, rest) = T::ref_from_prefix(remaining).map_err(|_e| Error::UnexpectedEof)?;
        self.advance(remaining.len() - rest.len());
        Ok(v)
    }

    /// Returns a typed slice of `count` elements of `T` and advances the reader past them.
    pub(crate) fn take_slice<T>(&mut self, count: usize) -> Result<&'a [T]>
    where
        T: zerocopy::FromBytes + zerocopy::KnownLayout + zerocopy::Immutable,
    {
        let remaining = self.remaining();
        let (slice, rest) = <[T]>::ref_from_prefix_with_elems(remaining, count)
            .map_err(|_e| Error::UnexpectedEof)?;
        self.advance(remaining.len() - rest.len());
        Ok(slice)
    }

    /// Returns a byte slice of `count` bytes and advances the reader past them.
    pub(crate) fn take_bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        let remaining = self.remaining();
        if remaining.len() < count {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &remaining[..count];
        self.advance(count);
        Ok(bytes)
    }
}
