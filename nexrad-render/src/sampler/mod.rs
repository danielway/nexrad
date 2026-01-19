//! Sampler abstraction for accessing radar data at arbitrary coordinates.
//!
//! This module provides a trait and implementations for sampling radar data
//! at world coordinates (meters from radar). Samplers are used internally
//! by the rendering functions.
//!
//! # Coordinate System
//!
//! World coordinates are in meters relative to the radar location:
//! - X is positive eastward
//! - Y is positive northward
//! - (0, 0) is the radar location

mod grid;
mod polar;

pub use grid::GridSampler;
pub use polar::PolarSampler;

/// Trait for sampling radar data at (x, y) coordinates in meters from radar.
///
/// Implementors provide access to radar data at arbitrary world coordinates,
/// handling coordinate transformation and bounds checking internally.
pub trait Sampler {
    /// Samples the data at the given world coordinates (meters from radar).
    ///
    /// Returns `Some(value)` if the coordinates are within bounds and the
    /// data at that location is valid (not NaN). Returns `None` if out of
    /// bounds or the data is invalid.
    fn sample(&self, x_m: f32, y_m: f32) -> Option<f32>;

    /// Returns the maximum extent in meters (for auto-scaling).
    ///
    /// For polar data, this is the maximum range.
    /// For grid data, this is the distance from center to the farthest corner.
    fn extent_m(&self) -> f32;
}
