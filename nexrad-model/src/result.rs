//! Result and Error types for NEXRAD model operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD model operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD model operations.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Attempted to merge sweeps with mismatched elevation numbers.
    ///
    /// This error occurs when calling [`Sweep::merge`](crate::data::Sweep::merge)
    /// with two sweeps that have different elevation numbers.
    #[error("two sweeps' elevation numbers do not match")]
    ElevationMismatch,

    /// Attempted to build a sweep field with invalid geometry.
    ///
    /// This error occurs when calling
    /// [`SweepFieldBuilder::build`](crate::data::SweepFieldBuilder::build) with no azimuths, a
    /// gate count of zero, or a non-positive gate interval.
    #[error("invalid sweep field geometry: {0}")]
    InvalidSweepGeometry(String),
}
