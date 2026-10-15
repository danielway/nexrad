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
//! - [`ScanDerivedProduct`] — produces a [`CartesianField`] on a [`CartesianGrid`] from
//!   the sweeps of a full scan (e.g. [`derived::CompositeReflectivity`])
//!
//! # Example
//!
//! ```
//! use nexrad_process::{SweepPipeline, filter::ThresholdFilter};
//! use nexrad_model::data::{SweepField, Product};
//! # use nexrad_model::data::{MomentData, Radial, Sweep};
//! # let reflectivity = MomentData::from_fixed_point(
//! #     Product::Reflectivity, 3, 2000, 250, 8, 2.0, 66.0, vec![100, 110, 120],
//! # )?;
//! # let radial = Radial::builder(0, 0.0, 1, 0.5).moment(reflectivity).build()?;
//! # let sweep = Sweep::new(1, vec![radial])?;
//!
//! let field = SweepField::from_radials(sweep.radials(), Product::Reflectivity).unwrap();
//! let filtered = SweepPipeline::new()
//!     .then(ThresholdFilter::new(Some(5.0), None)?)
//!     .execute(&field)?;
//! # Ok::<(), nexrad_process::Error>(())
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]

mod azimuth;
/// Derived products computed from radar scans.
pub mod derived;
/// Storm cell detection algorithms.
pub mod detection;
/// Filtering algorithms for sweep field data.
pub mod filter;
mod grid;
mod pipeline;
/// Result and error types for processing operations.
pub mod result;

pub use grid::CartesianGrid;
pub use pipeline::SweepPipeline;
pub use result::{Error, Result};

use nexrad_model::data::{CartesianField, SweepField};
use nexrad_model::geo::RadarCoordinateSystem;

/// Transforms one [`SweepField`] into another.
///
/// This is the primary processing trait for single-sweep algorithms such as
/// filtering, smoothing, and clutter removal. Processors are `Send + Sync`, so a
/// [`SweepPipeline`] can be shared across threads.
pub trait SweepProcessor: Send + Sync {
    /// A human-readable name for this processor.
    fn name(&self) -> &str;

    /// Process the input field, producing a new field with the same geometry.
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not suit the processor, for example when it holds a
    /// different product or geometry than the processor requires.
    fn process(&self, input: &SweepField) -> Result<SweepField>;
}

/// Produces a [`CartesianField`] from the sweeps of a full scan.
///
/// This trait is for derived products that combine data from multiple elevations
/// into a single geographic surface, such as composite reflectivity, echo tops,
/// and vertically integrated liquid (VIL).
pub trait ScanDerivedProduct: Send + Sync {
    /// A human-readable name for this product.
    fn name(&self) -> &str;

    /// Compute the derived product from the sweep fields of a scan.
    ///
    /// # Parameters
    ///
    /// - `fields` — Sweep fields for the relevant product, one per elevation
    /// - `coordinate_system` — Radar coordinate system for geographic projection
    /// - `grid` — Geographic extent and dimensions of the output field
    ///
    /// # Errors
    ///
    /// Returns an error if the fields do not suit the product, for example when none are given or
    /// they hold a different product than it requires.
    fn compute(
        &self,
        fields: &[SweepField],
        coordinate_system: &RadarCoordinateSystem,
        grid: &CartesianGrid,
    ) -> Result<CartesianField>;
}
