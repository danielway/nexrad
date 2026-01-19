//! Cartesian grid type for gridded radar data.

use super::GridSpec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Cartesian grid of radar-derived data.
///
/// This is the canonical interchange type for gridded radar data. Data is stored
/// in row-major order with the origin at the top-left corner.
///
/// # Coordinate System
///
/// - X increases eastward
/// - Y increases northward
/// - Origin is at top-left corner of grid
/// - Storage is row-major: `values[y * width + x]` where y=0 is the top row
///
/// # Example
///
/// ```
/// use nexrad_model::field::{CartesianGrid, GridSpec};
///
/// // Create a 100x100 grid with 250m pixels centered on radar
/// let spec = GridSpec::centered(100, 100, 250.0);
/// let values = vec![0.0f32; spec.pixel_count()];
/// let grid = CartesianGrid::from_spec(&spec, values);
///
/// assert_eq!(grid.width(), 100);
/// assert_eq!(grid.height(), 100);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CartesianGrid<T> {
    width: usize,
    height: usize,
    pixel_size_m: f32,
    origin_xy_m: (f32, f32),
    values: Vec<T>,
}

impl<T> CartesianGrid<T> {
    /// Creates a new CartesianGrid with the given geometry and data.
    ///
    /// # Arguments
    ///
    /// * `width` - Grid width in pixels
    /// * `height` - Grid height in pixels
    /// * `pixel_size_m` - Size of each pixel in meters
    /// * `origin_xy_m` - Top-left corner position in meters from radar (x, y)
    /// * `values` - Data values in row-major order
    ///
    /// # Panics
    ///
    /// Panics if `values.len() != width * height`.
    pub fn new(
        width: usize,
        height: usize,
        pixel_size_m: f32,
        origin_xy_m: (f32, f32),
        values: Vec<T>,
    ) -> Self {
        assert_eq!(
            values.len(),
            width * height,
            "values length {} does not match grid size {}x{}={}",
            values.len(),
            width,
            height,
            width * height
        );
        Self {
            width,
            height,
            pixel_size_m,
            origin_xy_m,
            values,
        }
    }

    /// Creates a CartesianGrid from a GridSpec.
    ///
    /// # Panics
    ///
    /// Panics if `values.len() != spec.pixel_count()`.
    pub fn from_spec(spec: &GridSpec, values: Vec<T>) -> Self {
        Self::new(
            spec.width,
            spec.height,
            spec.pixel_size_m,
            spec.origin_xy_m,
            values,
        )
    }

    /// Grid width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Grid height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Size of each pixel in meters.
    pub fn pixel_size_m(&self) -> f32 {
        self.pixel_size_m
    }

    /// Origin (top-left corner) in meters relative to radar (x_m, y_m).
    pub fn origin_xy_m(&self) -> (f32, f32) {
        self.origin_xy_m
    }

    /// Returns a reference to the data values.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Returns a mutable reference to the data values.
    pub fn values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    /// Consumes the grid and returns the values vector.
    pub fn into_values(self) -> Vec<T> {
        self.values
    }

    /// Returns the linear index for (x, y) pixel coordinates.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if x >= width or y >= height.
    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width, "x={} >= width={}", x, self.width);
        debug_assert!(y < self.height, "y={} >= height={}", y, self.height);
        y * self.width + x
    }

    /// Returns a reference to the value at (x, y).
    ///
    /// # Panics
    ///
    /// Panics if x >= width or y >= height.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.values[self.idx(x, y)]
    }

    /// Returns a mutable reference to the value at (x, y).
    ///
    /// # Panics
    ///
    /// Panics if x >= width or y >= height.
    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        let idx = self.idx(x, y);
        &mut self.values[idx]
    }

    /// Returns the center coordinates of the grid in meters relative to radar.
    pub fn center_xy_m(&self) -> (f32, f32) {
        (
            self.origin_xy_m.0 + (self.width as f32 * self.pixel_size_m) / 2.0,
            self.origin_xy_m.1 - (self.height as f32 * self.pixel_size_m) / 2.0,
        )
    }

    /// Converts pixel coordinates to world coordinates (meters from radar).
    ///
    /// Returns the center of the pixel in world coordinates.
    pub fn pixel_to_world(&self, x: usize, y: usize) -> (f32, f32) {
        (
            self.origin_xy_m.0 + (x as f32 + 0.5) * self.pixel_size_m,
            self.origin_xy_m.1 - (y as f32 + 0.5) * self.pixel_size_m,
        )
    }

    /// Converts world coordinates to pixel coordinates.
    ///
    /// Returns None if the coordinates are outside the grid bounds.
    pub fn world_to_pixel(&self, x_m: f32, y_m: f32) -> Option<(usize, usize)> {
        let px = ((x_m - self.origin_xy_m.0) / self.pixel_size_m).floor() as isize;
        let py = ((self.origin_xy_m.1 - y_m) / self.pixel_size_m).floor() as isize;

        if px >= 0 && py >= 0 && (px as usize) < self.width && (py as usize) < self.height {
            Some((px as usize, py as usize))
        } else {
            None
        }
    }

    /// Total number of pixels in the grid.
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cartesian_grid_creation() {
        let grid = CartesianGrid::new(100, 100, 250.0, (-12500.0, 12500.0), vec![0.0f32; 10000]);

        assert_eq!(grid.width(), 100);
        assert_eq!(grid.height(), 100);
        assert_eq!(grid.pixel_size_m(), 250.0);
        assert_eq!(grid.origin_xy_m(), (-12500.0, 12500.0));
    }

    #[test]
    fn test_cartesian_grid_from_spec() {
        let spec = GridSpec::centered(100, 100, 250.0);
        let grid = CartesianGrid::from_spec(&spec, vec![0.0f32; 10000]);

        assert_eq!(grid.width(), 100);
        assert_eq!(grid.height(), 100);
        // Centered grid should have center at (0, 0)
        let center = grid.center_xy_m();
        assert!((center.0).abs() < 0.01);
        assert!((center.1).abs() < 0.01);
    }

    #[test]
    fn test_cartesian_grid_indexing() {
        let grid = CartesianGrid::new(10, 10, 100.0, (0.0, 1000.0), (0..100).collect());

        assert_eq!(*grid.get(0, 0), 0);
        assert_eq!(*grid.get(9, 0), 9);
        assert_eq!(*grid.get(0, 1), 10);
        assert_eq!(*grid.get(5, 5), 55);
        assert_eq!(grid.idx(3, 2), 23);
    }

    #[test]
    fn test_pixel_to_world() {
        let grid = CartesianGrid::new(100, 100, 250.0, (-12500.0, 12500.0), vec![0.0f32; 10000]);

        // Top-left pixel center
        let (x, y) = grid.pixel_to_world(0, 0);
        assert!((x - (-12375.0)).abs() < 0.01); // -12500 + 0.5 * 250
        assert!((y - 12375.0).abs() < 0.01); // 12500 - 0.5 * 250

        // Center pixel
        let (x, y) = grid.pixel_to_world(50, 50);
        assert!((x - 125.0).abs() < 0.01);
        assert!((y - (-125.0)).abs() < 0.01);
    }

    #[test]
    fn test_world_to_pixel() {
        let grid = CartesianGrid::new(100, 100, 250.0, (-12500.0, 12500.0), vec![0.0f32; 10000]);

        // Center of grid
        assert_eq!(grid.world_to_pixel(0.0, 0.0), Some((50, 50)));

        // Top-left corner
        assert_eq!(grid.world_to_pixel(-12500.0, 12500.0), Some((0, 0)));

        // Out of bounds
        assert_eq!(grid.world_to_pixel(-20000.0, 0.0), None);
        assert_eq!(grid.world_to_pixel(20000.0, 0.0), None);
    }

    #[test]
    #[should_panic]
    fn test_cartesian_grid_size_mismatch() {
        let _ = CartesianGrid::new(100, 100, 250.0, (0.0, 0.0), vec![0.0f32; 50]);
    }
}
