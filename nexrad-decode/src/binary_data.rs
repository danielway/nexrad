//! Wrapper for binary data with a concise Debug representation.
//!
//! The `Debug` output shows the length, a 64-bit FNV-1a fingerprint, and the first and last four
//! bytes. FNV-1a is implemented inline so the fingerprint needs no dependency and is stable across
//! platforms and Rust versions, which keeps snapshot tests toolchain-independent.

use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// A wrapper for binary data that provides a concise Debug implementation showing size, a
/// 64-bit FNV-1a fingerprint, and a sample of head/tail bytes instead of the full binary content.
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct BinaryData<T>(pub T);

impl<T> BinaryData<T> {
    /// Creates a new BinaryData wrapper.
    pub fn new(data: T) -> Self {
        Self(data)
    }

    /// Unwraps the inner data.
    #[cfg(feature = "model")]
    pub fn into_inner(self) -> T {
        self.0
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

    #[test]
    fn test_debug_vec() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let binary_data = BinaryData::new(data);
        let debug_str = format!("{:?}", binary_data);

        assert!(debug_str.contains("len: 8"));
        assert!(debug_str.contains("fnv1a: 0x"));
        assert!(debug_str.contains("head: [1, 2, 3, 4]"));
        assert!(debug_str.contains("tail: [5, 6, 7, 8]"));
    }

    #[test]
    fn test_debug_array() {
        let data = [0xAA, 0xBB, 0xCC];
        let binary_data = BinaryData::new(data);
        let debug_str = format!("{:?}", binary_data);

        assert!(debug_str.contains("len: 3"));
        assert!(debug_str.contains("head: [170, 187, 204]"));
        assert!(debug_str.contains("tail: []")); // Less than 4 bytes
    }

    #[test]
    fn test_deref() {
        let data = vec![1, 2, 3];
        let binary_data = BinaryData::new(data);
        assert_eq!(binary_data.len(), 3);
        assert_eq!(&binary_data[0], &1);
    }
}
