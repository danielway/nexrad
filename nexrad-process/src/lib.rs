//! Processing algorithms for NEXRAD weather radar data.
//!
//! This crate provides processing traits and algorithms that operate on the field types
//! defined in [`nexrad_model`]. The core abstraction is the [`SweepProcessor`] trait,
//! which transforms one [`SweepField`] into another. Processors can be composed into
//! pipelines using [`SweepPipeline`].
//!
//! # Architecture
//!
//! Three processing traits cover different scopes:
//!
//! - [`SweepProcessor`] — single-sweep algorithms (filtering, smoothing)
//! - [`VolumeProcessor`] — multi-elevation algorithms (velocity dealiasing)
//! - [`VolumeDerivedProduct`] — produces a [`CartesianField`] from the full volume
//!   (composite reflectivity, VIL)
//!
//! # Example
//!
//! ```ignore
//! use nexrad_process::{SweepPipeline, filter::ThresholdFilter};
//! use nexrad_model::data::{SweepField, Product};
//!
//! let field = SweepField::from_radials(sweep.radials(), Product::Reflectivity).unwrap();
//! let filtered = SweepPipeline::new()
//!     .then(ThresholdFilter { min: Some(5.0), max: None })
//!     .execute(&field)?;
//! ```

pub mod derived;
pub mod filter;
pub mod pipeline;
pub mod result;

pub use pipeline::SweepPipeline;
pub use result::{Error, Result};

use nexrad_model::data::{CartesianField, Scan, SweepField};
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

/// Processes fields with full volume context (multiple elevations).
///
/// This trait is for algorithms that need to consider data across elevations,
/// such as velocity dealiasing.
pub trait VolumeProcessor {
    /// A human-readable name for this processor.
    fn name(&self) -> &str;

    /// Process sweep fields using the full volume context.
    ///
    /// Takes the scan metadata and all sweep fields for the relevant product,
    /// returning a new set of processed fields (one per input field).
    fn process_volume(&self, scan: &Scan, fields: &[SweepField]) -> Result<Vec<SweepField>>;
}

/// Produces a [`CartesianField`] from the full volume scan.
///
/// This trait is for derived products that combine data from multiple elevations
/// into a single geographic surface, such as composite reflectivity, echo tops,
/// and vertically integrated liquid (VIL).
pub trait VolumeDerivedProduct {
    /// A human-readable name for this product.
    fn name(&self) -> &str;

    /// Compute the derived product from the volume scan.
    ///
    /// # Parameters
    ///
    /// - `scan` — Volume scan metadata (site info, VCP, etc.)
    /// - `fields` — Sweep fields for the relevant product, one per elevation
    /// - `coord_system` — Radar coordinate system for geographic projection
    /// - `output_extent` — Geographic extent of the output grid
    /// - `output_resolution` — (width, height) of the output grid in cells
    fn compute(
        &self,
        scan: &Scan,
        fields: &[SweepField],
        coord_system: &RadarCoordinateSystem,
        output_extent: &GeoExtent,
        output_resolution: (usize, usize),
    ) -> Result<CartesianField>;
}
