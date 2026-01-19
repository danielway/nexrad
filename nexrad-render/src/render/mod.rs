//! Rendering functions for radar field data.
//!
//! This module provides the primary rendering API for the canonical interchange
//! types [`PolarSweep`] and [`CartesianGrid`].
//!
//! # Example
//!
//! ```ignore
//! use nexrad_model::field::PolarSweep;
//! use nexrad_render::render::{render_polar, RenderOpts};
//! use nexrad_render::get_nws_reflectivity_scale;
//! use piet_common::Device;
//!
//! let mut device = Device::new().unwrap();
//! let opts = RenderOpts::new(800, 800);
//! let image = render_polar(&mut device, &sweep, &get_nws_reflectivity_scale(), &opts)?;
//! image.save_to_file("output.png")?;
//! ```

mod grid;
mod image;
mod options;
mod polar;

pub use grid::render_grid;
pub use image::RenderedImage;
pub use options::RenderOpts;
pub use polar::render_polar;
