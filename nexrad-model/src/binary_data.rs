//! Wrapper for binary data with a concise Debug representation.
//!
//! The `Debug` output shows the length, a 64-bit FNV-1a fingerprint, and the first and last four
//! bytes. FNV-1a is implemented inline so the fingerprint needs no dependency and is stable across
//! platforms and Rust versions, which keeps snapshot tests toolchain-independent.

use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};

/// A wrapper for binary data that provides a concise Debug implementation showing size, a
/// 64-bit FNV-1a fingerprint, and a sample of head/tail bytes instead of the full binary content.
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BinaryData<T>(pub T);

impl<T> BinaryData<T> {
    /// Creates a new BinaryData wrapper.
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

/// Computes the 64-bit FNV-1a hash of `bytes`.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

impl<T: AsRef<[u8]>> Debug for BinaryData<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes = self.0.as_ref();
        let len = bytes.len();
        let hash = fnv1a_64(bytes);

        let head: Vec<u8> = bytes.iter().take(4).copied().collect();
        let tail: Vec<u8> = if len > 4 {
            bytes.iter().rev().take(4).rev().copied().collect()
        } else {
            Vec::new()
        };

        f.debug_struct("BinaryData")
            .field("len", &len)
            .field("fnv1a", &format_args!("0x{hash:016x}"))
            .field("head", &head)
            .field("tail", &tail)
            .finish()
    }
}

impl<T> Deref for BinaryData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for BinaryData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for BinaryData<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T> From<T> for BinaryData<T> {
    fn from(data: T) -> Self {
        Self(data)
    }
}

/// Serializes the wrapped bytes as a byte string, which compact formats encode as a length
/// prefix followed by the raw bytes.
#[cfg(feature = "serde")]
impl<T: AsRef<[u8]>> serde::Serialize for BinaryData<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_ref())
    }
}

/// Deserializes a byte string, or a sequence of integers for formats without a bytes type such
/// as JSON.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BinaryData<Vec<u8>> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_byte_buf(BytesVisitor)
            .map(BinaryData)
    }
}

/// Visits a byte string or a sequence of bytes.
#[cfg(feature = "serde")]
struct BytesVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a byte string or a sequence of bytes")
    }

    fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        Ok(bytes.to_vec())
    }

    fn visit_byte_buf<E: serde::de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
        Ok(bytes)
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(1 << 20));
        while let Some(byte) = seq.next_element::<u8>()? {
            bytes.push(byte);
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_known_vectors() {
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a_64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn debug_pins_fingerprint() {
        let binary_data = BinaryData::new(b"foobar".to_vec());
        assert_eq!(
            format!("{binary_data:?}"),
            "BinaryData { len: 6, fnv1a: 0x85944171f73967e8, head: [102, 111, 111, 98], \
             tail: [111, 98, 97, 114] }"
        );
    }
}
