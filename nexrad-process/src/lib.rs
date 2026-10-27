//! Processing algorithms for NEXRAD weather radar data.
//!
//! This crate provides processing traits and algorithms that operate on the field types
//! defined in [`nexrad_model`]. The core abstraction is the [`SweepProcessor`] trait,
//! which transforms one [`SweepField`] into another. Processors can be composed into
//! pipelines using [`SweepPipeline`].
//!
//! # Architecture
//!
//! - [`SweepProcessor`] — the trait for single-sweep algorithms (filtering, smoothing), which
//!   [`SweepPipeline`] chains
//! - [`derived`] — products computed from the sweeps of a full scan, each with its own inherent
//!   `compute` method, such as [`derived::CompositeReflectivity`] (a [`CartesianField`] on a
//!   [`CartesianGrid`]) and [`derived::VerticalCrossSection`]
//! - [`detection`] — connected-region detection on a sweep field
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
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]

// Compiles the README's code blocks as doc tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

mod azimuth;
/// Derived products computed from radar scans.
pub mod derived;
/// Connected-region detection on sweep fields.
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

#[cfg(doc)]
use nexrad_model::data::CartesianField;
use nexrad_model::data::SweepField;

/// Transforms one [`SweepField`] into another.
///
/// This is the primary processing trait for single-sweep algorithms such as
/// thresholding, smoothing and correlation coefficient masking. Processors are `Send + Sync`, so a
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
