//! Processing algorithms for NEXRAD weather radar data.
//!
//! This crate provides processing traits and algorithms that operate on the field types
//! defined in [`nexrad_model`]. The core abstraction is the [`SweepProcessor`] trait,
//! which transforms one [`SweepField`] into another. Processors can be composed into
//! pipelines using [`SweepPipeline`].
//!
//! # Architecture
//!
//! Two processing traits cover different scopes:
//!
//! - [`SweepProcessor`] — single-sweep algorithms (filtering, smoothing)
//! - [`ScanDerivedProduct`] — produces a [`CartesianField`] from the sweeps of a full
//!   scan (e.g. [`derived::CompositeReflectivity`])
//!
//! # Example
//!
//! ```ignore
//! use nexrad_process::{SweepPipeline, filter::ThresholdFilter};
//! use nexrad_model::data::{SweepField, Product};
//!
//! let field = SweepField::from_radials(sweep.radials(), Product::Reflectivity).unwrap();
//! let filtered = SweepPipeline::new()
//!     .then(ThresholdFilter::new(Some(5.0), None)?)
//!     .execute(&field)?;
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]

/// Derived products computed from radar scans.
pub mod derived;
/// Storm cell detection algorithms.
pub mod detection;
/// Filtering algorithms for sweep field data.
pub mod filter;
/// Composable processing pipelines.
pub mod pipeline;
/// Result and error types for processing operations.
pub mod result;

pub use pipeline::SweepPipeline;
pub use result::{Error, Result};

use nexrad_model::data::{CartesianField, SweepField};
use nexrad_model::geo::{GeoExtent, RadarCoordinateSystem};

/// Transforms one [`SweepField`] into another.
///
/// This is the primary processing trait for single-sweep algorithms such as
/// filtering, smoothing, and clutter removal.
pub trait SweepProcessor {
    /// A human-readable name for this processor.
    fn name(&self) -> &str;

    /// Process the input field, producing a new field with the same geometry.
    fn process(&self, input: &SweepField) -> Result<SweepField>;
}

/// Produces a [`CartesianField`] from the sweeps of a full scan.
///
/// This trait is for derived products that combine data from multiple elevations
/// into a single geographic surface, such as composite reflectivity, echo tops,
/// and vertically integrated liquid (VIL).
pub trait ScanDerivedProduct {
    /// A human-readable name for this product.
    fn name(&self) -> &str;

    /// Compute the derived product from the sweep fields of a scan.
    ///
    /// # Parameters
    ///
    /// - `fields` — Sweep fields for the relevant product, one per elevation
    /// - `coord_system` — Radar coordinate system for geographic projection
    /// - `output_extent` — Geographic extent of the output grid
    /// - `output_resolution` — (width, height) of the output grid in cells
    fn compute(
        &self,
        fields: &[SweepField],
        coord_system: &RadarCoordinateSystem,
        output_extent: &GeoExtent,
        output_resolution: (usize, usize),
    ) -> Result<CartesianField>;
}
