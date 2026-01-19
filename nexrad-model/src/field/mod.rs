//! Canonical interchange types for radar field data.
//!
//! This module provides uniform data structures for representing radar data
//! in both polar and Cartesian coordinate systems. These types are designed
//! for interchange between processing stages and rendering.
//!
//! # Types
//!
//! - [`PolarSweep<T>`] - Polar coordinate radar sweep data
//! - [`CartesianGrid<T>`] - Cartesian gridded radar data
//! - [`GridSpec`] - Specification for grid geometry
//!
//! # Conversion
//!
//! The [`radials_to_polar_sweep`] function converts from the decode-level
//! [`Radial`](crate::data::Radial) type to the interchange [`PolarSweep`] type.
//!
//! # Invalid Values
//!
//! Invalid data (below threshold, range folded, etc.) is represented as
//! `f32::NAN` in the values arrays. Consumers should check for NaN when
//! processing data.
//!
//! # Example
//!
//! ```ignore
//! use nexrad_model::field::{PolarSweep, ProductSelector, radials_to_polar_sweep};
//! use nexrad_model::data::Radial;
//!
//! let radials: &[Radial] = /* ... */;
//! let sweep = radials_to_polar_sweep(radials, ProductSelector::Reflectivity)?;
//!
//! println!("Rays: {}, Gates: {}", sweep.ray_count(), sweep.gate_count());
//!
//! // Iterate over rays
//! for (azimuth, gate_values) in sweep.rays() {
//!     println!("Azimuth {}: {} gates", azimuth, gate_values.len());
//! }
//! ```

mod cartesian_grid;
mod convert;
mod error;
mod grid_spec;
mod polar_sweep;

pub use cartesian_grid::CartesianGrid;
pub use convert::{radials_to_polar_sweep, ProductSelector};
pub use error::FieldError;
pub use grid_spec::GridSpec;
pub use polar_sweep::PolarSweep;
