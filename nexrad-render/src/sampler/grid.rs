//! Cartesian grid sampler.

use super::Sampler;
use nexrad_model::field::CartesianGrid;

/// Sampler for Cartesian grid data.
///
/// Converts (x, y) world coordinates to pixel coordinates and returns
/// the value at that location.
pub struct GridSampler<'a> {
    grid: &'a CartesianGrid<f32>,
}

impl<'a> GridSampler<'a> {
    /// Creates a new sampler for the given Cartesian grid.
    pub fn new(grid: &'a CartesianGrid<f32>) -> Self {
        Self { grid }
    }

    /// Returns a reference to the underlying grid.
    pub fn grid(&self) -> &'a CartesianGrid<f32> {
        self.grid
    }
}

impl<'a> Sampler for GridSampler<'a> {
    fn sample(&self, x_m: f32, y_m: f32) -> Option<f32> {
        // Convert world coordinates to pixel coordinates
        let (px, py) = self.grid.world_to_pixel(x_m, y_m)?;

        // Get value and check for NaN
        let value = *self.grid.get(px, py);
        if value.is_nan() {
            return None;
        }

        Some(value)
    }

    fn extent_m(&self) -> f32 {
        let origin = self.grid.origin_xy_m();
        let pixel_size = self.grid.pixel_size_m();

        // Find the maximum distance from (0,0) to any corner of the grid
        let corner_x = origin
            .0
            .abs()
            .max((origin.0 + self.grid.width() as f32 * pixel_size).abs());
        let corner_y = origin
            .1
            .abs()
            .max((origin.1 - self.grid.height() as f32 * pixel_size).abs());

        (corner_x * corner_x + corner_y * corner_y).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::field::GridSpec;

    #[test]
    fn test_grid_sampler_center() {
        let spec = GridSpec::centered(100, 100, 250.0);
        let grid = CartesianGrid::from_spec(&spec, vec![42.0f32; 10000]);
        let sampler = GridSampler::new(&grid);

        // Center should be valid
        let result = sampler.sample(0.0, 0.0);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_grid_sampler_out_of_bounds() {
        let spec = GridSpec::centered(100, 100, 250.0);
        let grid = CartesianGrid::from_spec(&spec, vec![42.0f32; 10000]);
        let sampler = GridSampler::new(&grid);

        // Way outside should be None
        assert!(sampler.sample(100000.0, 0.0).is_none());
        assert!(sampler.sample(-100000.0, 0.0).is_none());
    }

    #[test]
    fn test_grid_sampler_nan_value() {
        let spec = GridSpec::centered(100, 100, 250.0);
        let mut values = vec![42.0f32; 10000];
        // Set center pixel to NaN
        values[50 * 100 + 50] = f32::NAN;

        let grid = CartesianGrid::from_spec(&spec, values);
        let sampler = GridSampler::new(&grid);

        // Center should now be None (NaN)
        let result = sampler.sample(0.0, 0.0);
        assert!(result.is_none());
    }
}
