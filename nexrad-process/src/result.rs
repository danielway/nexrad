//! Result and error types for NEXRAD processing operations.

/// Errors that can occur during processing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The input field has invalid geometry for this algorithm.
    #[error("invalid field geometry: {0}")]
    InvalidGeometry(String),

    /// The algorithm requires data that is not present.
    #[error("missing required data: {0}")]
    MissingData(String),

    /// A parameter value is out of the acceptable range.
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// A model constructor or accessor rejected the output geometry, such as a composite
    /// extent outside valid latitudes and longitudes.
    #[error("model error: {0}")]
    Model(#[from] nexrad_model::result::Error),
}

/// Result type for processing operations.
pub type Result<T> = std::result::Result<T, Error>;
