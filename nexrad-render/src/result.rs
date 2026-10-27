//! Result and error types for NEXRAD rendering operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD rendering operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD rendering.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The field to render contains no data (no radials, gates, rows or columns).
    #[error("field contains no data")]
    EmptyField,
    /// The output image dimensions were invalid (zero, or too large for an image
    /// buffer).
    #[error("invalid image dimensions")]
    InvalidDimensions,
    /// A value range was not finite and ordered (`min < max`), for example the range
    /// passed to [`ColorLookupTable::new`](crate::ColorLookupTable::new) or
    /// [`RenderOptions::with_value_range`](crate::RenderOptions::with_value_range).
    #[error("invalid value range {min}..{max}: must be finite with min < max")]
    #[non_exhaustive]
    InvalidRange {
        /// The lower bound that was given.
        min: f32,
        /// The upper bound that was given.
        max: f32,
    },
    /// A color lookup table was requested with fewer than 2 entries.
    #[error("invalid color lookup table size {size}: must be at least 2")]
    #[non_exhaustive]
    InvalidTableSize {
        /// The size that was given.
        size: usize,
    },
    /// A color scale was given no levels or stops, or a value that is not finite, by
    /// [`DiscreteColorScale::new`](crate::DiscreteColorScale::new) or
    /// [`ContinuousColorScale::new`](crate::ContinuousColorScale::new).
    #[error("invalid color scale: {reason}")]
    #[non_exhaustive]
    InvalidColorScale {
        /// Why the scale was rejected.
        reason: &'static str,
    },
    /// A geographic extent was not finite and ordered (`min < max` in both latitude
    /// and longitude).
    #[error("invalid geographic extent: must be finite with min < max")]
    InvalidExtent,
    /// A sweep was rendered into a geographic extent without a radar coordinate system
    /// to place it; set one with
    /// [`RenderOptions::with_coord_system`](crate::RenderOptions::with_coord_system).
    #[error("rendering into a geographic extent requires a coordinate system")]
    MissingCoordinateSystem,
    /// Writing an encoded image failed, for example because the output file could not
    /// be created.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    /// The image could not be encoded. The source is the encoder's error.
    #[error("error encoding image")]
    Encode(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}
