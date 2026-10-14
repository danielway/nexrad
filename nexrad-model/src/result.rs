//! Result and Error types for NEXRAD model operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD model operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD model operations.
#[derive(ThisError, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A radial or sweep has a different elevation number than the sweep it is placed in.
    ///
    /// Returned by [`Sweep::new`](crate::data::Sweep::new) when a radial's elevation number
    /// differs from the sweep's, and by [`Sweep::merge`](crate::data::Sweep::merge) when the two
    /// sweeps' elevation numbers differ.
    #[error("elevation number mismatch: expected {expected}, got {actual}")]
    ElevationMismatch {
        /// The elevation number of the sweep.
        expected: u8,
        /// The elevation number of the offending radial or sweep.
        actual: u8,
    },

    /// A field or radial geometry parameter is invalid.
    ///
    /// Returned by [`CartesianField::new`](crate::data::CartesianField::new),
    /// [`VerticalField::new`](crate::data::VerticalField::new),
    /// [`SweepFieldBuilder::build`](crate::data::SweepFieldBuilder::build), and
    /// [`RadialBuilder::build`](crate::data::RadialBuilder::build).
    #[error("invalid geometry: `{parameter}` {reason}")]
    InvalidGeometry {
        /// The name of the offending parameter.
        parameter: &'static str,
        /// Why the parameter's value was rejected.
        reason: &'static str,
    },

    /// A moment data encoding parameter is invalid, such as an unsupported `data_word_size` or a
    /// non-finite `scale`.
    ///
    /// Returned by [`MomentData::from_fixed_point`](crate::data::MomentData::from_fixed_point).
    #[error("invalid moment data: `{parameter}` {reason}")]
    InvalidMomentData {
        /// The name of the offending parameter.
        parameter: &'static str,
        /// Why the parameter's value was rejected.
        reason: &'static str,
    },

    /// Moment data's encoded values do not hold exactly the declared number of gates.
    ///
    /// Returned by [`MomentData::from_fixed_point`](crate::data::MomentData::from_fixed_point).
    #[error("invalid moment data length: expected {expected} bytes for the gate count and word size, got {actual}")]
    MomentDataLength {
        /// The number of bytes `gate_count` gates of `data_word_size` bits occupy.
        expected: usize,
        /// The number of bytes provided.
        actual: usize,
    },

    /// Radar site metadata is invalid.
    ///
    /// Returned by [`SiteBuilder::build`](crate::meta::SiteBuilder::build).
    #[error("invalid site: `{parameter}` {reason}")]
    InvalidSite {
        /// The name of the offending parameter.
        parameter: &'static str,
        /// Why the parameter's value was rejected.
        reason: &'static str,
    },

    /// A row or column index is outside a field's dimensions.
    ///
    /// Returned by the field `set` methods, such as
    /// [`SweepField::set`](crate::data::SweepField::set).
    #[error(
        "index ({row}, {column}) out of bounds for a field of {rows} rows and {columns} columns"
    )]
    IndexOutOfBounds {
        /// The requested row index.
        row: usize,
        /// The requested column index.
        column: usize,
        /// The number of rows in the field.
        rows: usize,
        /// The number of columns in the field.
        columns: usize,
    },
}
