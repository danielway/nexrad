//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped images that can
//! be saved to common formats like PNG.
//!
//! # Example
//!
//! ```ignore
//! use nexrad_model::data::Product;
//! use nexrad_render::{render_radials, RenderOptions, nws_reflectivity_scale};
//!
//! let options = RenderOptions::new(800, 800);
//! let image = render_radials(
//!     sweep.radials(),
//!     Product::Reflectivity,
//!     &nws_reflectivity_scale(),
//!     &options,
//! ).unwrap();
//!
//! // Save directly to PNG
//! image.save("radar.png").unwrap();
//! ```
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
//! - Consume `nexrad-model` types (Radial, MomentData)
//!
//! ## Constraints
//!
//! - **No data access or network operations**
//! - **No binary parsing or decoding**
//!
//! This crate can be used standalone or through the `nexrad` facade crate (re-exported
//! via the `render` feature, which is enabled by default).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]

pub use image::RgbaImage;
pub use nexrad_model::data::Product;

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
