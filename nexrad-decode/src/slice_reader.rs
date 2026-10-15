use crate::result::{Error, Result};

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
    /// Creates a SliceReader starting at `pos`.
    pub fn at(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    /// Returns the current byte position in the slice.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the remaining unread bytes.
    /// Returns an empty slice if position is at or past the end.
    pub fn remaining(&self) -> &'a [u8] {
        if self.pos >= self.data.len() {
            &[]
        } else {
            &self.data[self.pos..]
        }
    }

    /// Advances the position by `n` bytes.
    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// Peeks at the next two bytes as a big-endian u16 without advancing.
    pub fn peek_u16(&self) -> Option<u16> {
        let r = self.remaining();
        (r.len() >= 2).then(|| u16::from_be_bytes([r[0], r[1]]))
    }

    /// Returns a typed reference to the next `T` and advances the reader past it.
    pub(crate) fn take_ref<T>(&mut self) -> Result<&'a T>
    where
        T: zerocopy::FromBytes + zerocopy::KnownLayout + zerocopy::Immutable,
    {
        let remaining = self.remaining();
        let (v, rest) = T::ref_from_prefix(remaining)
            .map_err(|_e| Error::UnexpectedEof { offset: self.pos })?;
        self.advance(remaining.len() - rest.len());
        Ok(v)
    }

    /// Returns a byte slice of `count` bytes and advances the reader past them.
    pub(crate) fn take_bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        let remaining = self.remaining();
        if remaining.len() < count {
            return Err(Error::UnexpectedEof { offset: self.pos });
        }
        let bytes = &remaining[..count];
        self.advance(count);
        Ok(bytes)
    }
}
