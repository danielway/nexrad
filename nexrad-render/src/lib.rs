//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped images that can
//! be saved to common formats like PNG.
//!
//! # Example
//!
//! Extract a [`SweepField`](nexrad_model::data::SweepField) from a sweep (or build one
//! with `SweepField::from_radials`), pick a color scale and render it:
//!
//! ```no_run
//! use nexrad_model::data::Sweep;
//! use nexrad_model::data::Product;
//! use nexrad_render::{default_scale, render_sweep, RenderOptions};
//!
//! fn render(sweep: &Sweep) -> nexrad_render::Result<()> {
//!     let Some(field) = sweep.field(Product::Reflectivity) else {
//!         return Ok(()); // this sweep carries no reflectivity
//!     };
//!
//!     let scale = default_scale(Product::Reflectivity);
//!     let result = render_sweep(&field, &scale, &RenderOptions::new(800, 800))?;
//!
//!     // Save directly to PNG
//!     result.save("radar.png")?;
//!
//!     // Metadata maps pixels back to radar coordinates
//!     let ring_px = result.metadata().km_to_pixel_distance(100.0);
//!     println!("100 km ring radius: {ring_px:?} px");
//!     Ok(())
//! }
//! ```
//!
//! [`render_cartesian`] and [`render_vertical`] render gridded and cross-section fields
//! the same way; [`RenderMetadata::projection`] reports how each image's pixels map
//! back to data coordinates.
//!
//! # The `image` dependency
//!
//! Rendered images are [`image::RgbaImage`] values and image I/O failures surface as
//! [`Error::Image`], so the [`image`] crate is part of this crate's public API. It is
//! re-exported as [`nexrad_render::image`](crate::image) so callers can use the
//! matching version without a separate dependency. A major version bump of `image`
//! is therefore a major version bump of `nexrad-render`.
//!
//! Only the `image` PNG codec is enabled. [`RenderResult::save`] and
//! [`RenderResult::write_to`] support other formats once the consumer enables the
//! matching feature on its own `image` dependency (Cargo unifies the features).
//!
//! # Crate Boundaries
//!
//! This crate provides **visualization and rendering** with the following responsibilities
//! and constraints:
//!
//! ## Responsibilities
//!
//! - Render radar data to images ([`image::RgbaImage`])
//! - Apply color scales to moment data
//! - Handle geometric transformations (polar to Cartesian coordinates)
//! - Consume `nexrad-model` field types (`SweepField`, `CartesianField`, `VerticalField`)
//!
//! ## Constraints
//!
//! - **No data access or network operations**
//! - **No binary parsing or decoding**
//!
//! This crate can be used standalone or through the `nexrad` facade crate (re-exported
//! via the `render` feature, which is enabled by default).

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

pub use image;
pub use image::RgbaImage;

mod color;
pub use crate::color::*;

mod grid;
pub use grid::{render_cartesian, render_vertical};

mod options;
pub use options::{Interpolation, RenderOptions};

mod render_result;
pub use render_result::{PointQuery, Projection, RenderMetadata, RenderResult};

pub mod result;
pub use result::{Error, Result};

mod sweep;
pub use sweep::render_sweep;
