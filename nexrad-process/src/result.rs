//! Result and error types for NEXRAD processing operations.

use nexrad_model::data::{Product, SweepField};

/// Errors that can occur during processing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A constructor or builder parameter is outside its accepted range.
    #[error("invalid parameter: `{parameter}` {reason}")]
    InvalidParameter {
        /// The name of the offending parameter.
        parameter: &'static str,
        /// Why the parameter's value was rejected.
        reason: &'static str,
    },

    /// An input field or output grid has invalid geometry for this algorithm.
    #[error("invalid geometry: `{parameter}` {reason}")]
    InvalidGeometry {
        /// The name of the offending geometry parameter.
        parameter: &'static str,
        /// Why the geometry was rejected.
        reason: &'static str,
    },

    /// Two fields that must share a polar geometry differ in one of its parameters.
    ///
    /// Returned by
    /// [`CorrelationCoefficientFilter`](crate::filter::CorrelationCoefficientFilter) when the
    /// input field and the correlation coefficient field are not registered gate for gate.
    #[error("geometry mismatch: `{parameter}` differs between the input and the reference field")]
    GeometryMismatch {
        /// The name of the [`SweepField`] geometry accessor whose values differ.
        parameter: &'static str,
    },

    /// A field holds a different radar product than the algorithm requires.
    #[error("unexpected product: expected {expected:?}, got {actual:?}")]
    UnexpectedProduct {
        /// The product the algorithm requires.
        expected: Product,
        /// The product the field holds, if any.
        actual: Option<Product>,
    },

    /// The algorithm requires data that is not present.
    #[error("missing required data: {what}")]
    MissingData {
        /// A description of the missing data.
        what: &'static str,
    },

    /// A model constructor or accessor rejected the output geometry, such as a composite
    /// extent outside valid latitudes and longitudes.
    #[error("model error")]
    Model(#[from] nexrad_model::result::Error),
    /// A failure reported by a [`SweepProcessor`](crate::SweepProcessor) implemented outside
    /// this crate, such as an I/O error while loading its parameters.
    #[error(transparent)]
    Custom(Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Result type for processing operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Check that `field` holds the `expected` product.
pub(crate) fn check_product(field: &SweepField, expected: Product) -> Result<()> {
    match field.product() {
        Some(actual) if actual == expected => Ok(()),
        actual => Err(Error::UnexpectedProduct { expected, actual }),
    }
}

/// Check that an `f32` parameter is finite.
pub(crate) fn check_finite(parameter: &'static str, value: f32) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidParameter {
            parameter,
            reason: "must be finite",
        })
    }
}
