//! Error types for field data operations.

use thiserror::Error as ThisError;

/// Errors that can occur during field data operations.
#[derive(ThisError, Debug, Clone, PartialEq)]
pub enum FieldError {
    /// No radials provided for conversion.
    #[error("no radials provided")]
    EmptyInput,

    /// Radials have inconsistent gate counts.
    #[error("inconsistent gate counts: expected {expected}, found {found} at ray {ray_index}")]
    InconsistentGateCount {
        /// Expected gate count from first radial.
        expected: u16,
        /// Found gate count at the specified ray.
        found: u16,
        /// Index of the ray with inconsistent gate count.
        ray_index: usize,
    },

    /// Radials have inconsistent gate intervals.
    #[error(
        "inconsistent gate intervals: expected {expected} m, found {found} m at ray {ray_index}"
    )]
    InconsistentGateInterval {
        /// Expected gate interval from first radial (meters).
        expected: f32,
        /// Found gate interval at the specified ray (meters).
        found: f32,
        /// Index of the ray with inconsistent interval.
        ray_index: usize,
    },

    /// Radials have inconsistent first gate ranges.
    #[error(
        "inconsistent first gate ranges: expected {expected} m, found {found} m at ray {ray_index}"
    )]
    InconsistentFirstGateRange {
        /// Expected first gate range from first radial (meters).
        expected: f32,
        /// Found first gate range at the specified ray (meters).
        found: f32,
        /// Index of the ray with inconsistent range.
        ray_index: usize,
    },

    /// The requested product is not available in the radial data.
    #[error("product not available in radial data")]
    ProductNotAvailable,

    /// Values array size does not match geometry.
    #[error("values array size mismatch: expected {expected}, got {actual}")]
    ValuesSizeMismatch {
        /// Expected number of values based on geometry.
        expected: usize,
        /// Actual number of values provided.
        actual: usize,
    },
}
