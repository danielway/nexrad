/// A volume's index in the AWS real-time NEXRAD bucket. These indexes are rotated-through as chunks
/// are accumulated and finally combined into full volumes to be archived. Indexes range over
/// `1..=999`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VolumeIndex(usize);

impl VolumeIndex {
    /// Creates a new volume index with the specified value.
    ///
    /// Valid indexes are `1..=999`, matching the bucket's volume directories. Passing a value
    /// outside that range is a logic error: it panics in debug builds and produces an index that
    /// names no volume in release builds.
    pub fn new(index: usize) -> Self {
        debug_assert!(
            (1..=999).contains(&index),
            "Volume index must be in 1..=999, got {index}"
        );
        Self(index)
    }

    /// Returns the volume index as a number.
    pub fn as_number(&self) -> usize {
        self.0
    }

    /// Returns the next volume index, wrapping from 999 back to 1.
    pub fn next(&self) -> Self {
        if self.0 == 999 {
            Self::new(1)
        } else {
            Self::new(self.0 + 1)
        }
    }
}
