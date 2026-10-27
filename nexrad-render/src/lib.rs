//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped RGBA pixel
//! buffers that can be uploaded as textures, handed to another image library, or
//! encoded as PNG.
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
//!     // Raw RGBA8 pixels, row-major
//!     let (width, height) = (result.width(), result.height());
//!     assert_eq!(result.pixels().len(), (width * height * 4) as usize);
//!
//!     // Or encode as PNG
//!     result.save_png("radar.png")?;
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
//! # Pixels and encoding
//!
//! A [`RenderResult`] owns its pixels as a `Vec<u8>` of 8-bit RGBA, row-major with no
//! row padding ([`RenderResult::pixels`], [`RenderResult::into_pixels`]). No image
//! library type appears in this crate's API. [`RenderResult::write_png`] and
//! [`RenderResult::save_png`] encode PNG with a private encoder; for other formats,
//! pass the pixels to the image library of your choice.
//!
//! # Features
//!
//! - `image` (off by default): adds `RenderResult::into_rgba_image`, which converts
//!   the result into an `image::RgbaImage`. This feature tracks the `image` crate's own
//!   0.x releases and is excluded from this crate's semver promise: a minor or patch
//!   release of this crate may move it to a newer, incompatible `image` version.
//!
//! # Crate Boundaries
//!
//! This crate renders `nexrad-model` field types (`SweepField`, `CartesianField`,
//! `VerticalField`) to RGBA pixel buffers, applying color scales and the polar-to-Cartesian
//! mapping. It does no data access, network I/O or binary decoding. It is re-exported by the
//! `nexrad` facade through the `render` feature, which is on by default.

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
