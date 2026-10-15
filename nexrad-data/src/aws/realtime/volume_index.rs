use crate::result::{AwsError, Error, Result};

/// A volume's index in the AWS real-time NEXRAD bucket. These indexes are rotated-through as chunks
/// are accumulated and finally combined into full volumes to be archived. Indexes range over
/// `1..=999`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VolumeIndex(usize);

impl VolumeIndex {
    /// The first volume index.
    const FIRST: usize = 1;

    /// The last volume index.
    const LAST: usize = 999;

    /// Creates a volume index, validating that it is in `1..=999`, the bucket's volume
    /// directories.
    ///
    /// # Errors
    ///
    /// Returns [`AwsError::InvalidVolumeIndex`] for any other value.
    pub fn new(index: usize) -> Result<Self> {
        if (Self::FIRST..=Self::LAST).contains(&index) {
            Ok(Self(index))
        } else {
            Err(Error::Aws(AwsError::InvalidVolumeIndex { index }))
        }
    }

    /// Returns the volume index as a number.
    pub fn as_number(&self) -> usize {
        self.0
    }

    /// Returns the next volume index, wrapping from 999 back to 1.
    pub fn next(&self) -> Self {
        if self.0 >= Self::LAST {
            Self(Self::FIRST)
        } else {
            Self(self.0 + 1)
        }
    }
}
