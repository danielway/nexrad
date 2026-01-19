//! Grid specification for Cartesian grids.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Specification for a Cartesian grid geometry.
///
/// This is used to define the output grid for resampling operations.
/// The origin is at the top-left corner, with X increasing eastward
/// and Y increasing southward (matching image conventions).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GridSpec {
    /// Grid width in pixels.
    pub width: usize,
    /// Grid height in pixels.
    pub height: usize,
    /// Size of each pixel in meters.
    pub pixel_size_m: f32,
    /// Origin (top-left corner) in meters relative to radar (x_m, y_m).
    /// X is positive eastward, Y is positive northward.
    pub origin_xy_m: (f32, f32),
}

impl GridSpec {
    /// Creates a new GridSpec with explicit origin.
    ///
    /// # Arguments
    ///
    /// * `width` - Grid width in pixels
    /// * `height` - Grid height in pixels
    /// * `pixel_size_m` - Size of each pixel in meters
    /// * `origin_xy_m` - Top-left corner position in meters from radar (x, y)
    pub fn new(width: usize, height: usize, pixel_size_m: f32, origin_xy_m: (f32, f32)) -> Self {
        Self {
            width,
            height,
            pixel_size_m,
            origin_xy_m,
        }
    }

    /// Creates a GridSpec centered on the radar.
    ///
    /// The grid will be centered at (0, 0) with the origin computed
    /// such that the center of the grid is at the radar location.
    ///
    /// # Arguments
    ///
    /// * `width` - Grid width in pixels
    /// * `height` - Grid height in pixels
    /// * `pixel_size_m` - Size of each pixel in meters
    pub fn centered(width: usize, height: usize, pixel_size_m: f32) -> Self {
        let half_width = (width as f32 * pixel_size_m) / 2.0;
        let half_height = (height as f32 * pixel_size_m) / 2.0;
        Self {
            width,
            height,
            pixel_size_m,
            origin_xy_m: (-half_width, half_height),
        }
    }

    /// Total number of pixels in the grid.
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }

    /// Returns the center coordinates of the grid in meters relative to radar.
    pub fn center_xy_m(&self) -> (f32, f32) {
        (
            self.origin_xy_m.0 + (self.width as f32 * self.pixel_size_m) / 2.0,
            self.origin_xy_m.1 - (self.height as f32 * self.pixel_size_m) / 2.0,
        )
    }
}
