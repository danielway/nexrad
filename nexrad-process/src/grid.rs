use crate::result::{Error, Result};
use nexrad_model::geo::GeoExtent;

/// The geographic output grid of a derived product such as
/// [`CompositeReflectivity`](crate::derived::CompositeReflectivity).
///
/// The grid covers `extent` with `width` columns and `height` rows, matching the layout of the
/// [`CartesianField`](nexrad_model::data::CartesianField) the product returns: row 0 is the
/// north edge and column 0 the west edge.
///
/// # Example
///
/// ```
/// use nexrad_model::geo::{GeoExtent, GeoPoint};
/// use nexrad_process::CartesianGrid;
///
/// let extent = GeoExtent {
///     min: GeoPoint { latitude: 34.0, longitude: -99.0 },
///     max: GeoPoint { latitude: 36.0, longitude: -97.0 },
/// };
/// let grid = CartesianGrid::new(extent, 800, 800)?;
/// assert_eq!(grid.width(), 800);
/// # Ok::<(), nexrad_process::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CartesianGrid {
    extent: GeoExtent,
    width: usize,
    height: usize,
}

impl CartesianGrid {
    /// Create an output grid of `width` columns by `height` rows over `extent`.
    ///
    /// The rules match [`CartesianField::new`](nexrad_model::data::CartesianField::new).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGeometry`] if `width` or `height` is zero or their product
    /// overflows, or the extent is not finite, within valid latitudes and longitudes, and
    /// ordered (`min < max`).
    pub fn new(extent: GeoExtent, width: usize, height: usize) -> Result<Self> {
        if width == 0 {
            return Err(invalid_geometry("width", "must be greater than zero"));
        }
        if height == 0 {
            return Err(invalid_geometry("height", "must be greater than zero"));
        }
        if width.checked_mul(height).is_none() {
            return Err(invalid_geometry(
                "width",
                "times the height overflows usize",
            ));
        }
        check_coordinate_range(
            "extent.latitude",
            (extent.min.latitude, extent.max.latitude),
            90.0,
            "must be within -90.0..=90.0",
        )?;
        check_coordinate_range(
            "extent.longitude",
            (extent.min.longitude, extent.max.longitude),
            180.0,
            "must be within -180.0..=180.0",
        )?;
        Ok(Self {
            extent,
            width,
            height,
        })
    }

    /// Geographic extent of the grid.
    pub fn extent(&self) -> &GeoExtent {
        &self.extent
    }

    /// Number of columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of rows.
    pub fn height(&self) -> usize {
        self.height
    }
}

fn invalid_geometry(parameter: &'static str, reason: &'static str) -> Error {
    Error::InvalidGeometry { parameter, reason }
}

/// Check that a `(min, max)` coordinate range is finite, ordered, and within `-limit..=limit`.
fn check_coordinate_range(
    parameter: &'static str,
    (min, max): (f64, f64),
    limit: f64,
    out_of_range_reason: &'static str,
) -> Result<()> {
    if !(min.is_finite() && max.is_finite()) {
        return Err(invalid_geometry(parameter, "must be finite"));
    }
    if min >= max {
        return Err(invalid_geometry(parameter, "must have min less than max"));
    }
    let bounds = -limit..=limit;
    if bounds.contains(&min) && bounds.contains(&max) {
        Ok(())
    } else {
        Err(invalid_geometry(parameter, out_of_range_reason))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::geo::GeoPoint;

    fn extent(min: (f64, f64), max: (f64, f64)) -> GeoExtent {
        GeoExtent {
            min: GeoPoint {
                latitude: min.0,
                longitude: min.1,
            },
            max: GeoPoint {
                latitude: max.0,
                longitude: max.1,
            },
        }
    }

    #[test]
    fn test_grid_accessors() -> Result<()> {
        let e = extent((34.0, -99.0), (36.0, -97.0));
        let grid = CartesianGrid::new(e, 40, 20)?;
        assert_eq!(grid.extent(), &e);
        assert_eq!(grid.width(), 40);
        assert_eq!(grid.height(), 20);
        Ok(())
    }

    #[test]
    fn test_grid_rejects_zero_dimensions() {
        let e = extent((34.0, -99.0), (36.0, -97.0));
        assert_eq!(
            CartesianGrid::new(e, 0, 10),
            Err(invalid_geometry("width", "must be greater than zero"))
        );
        assert_eq!(
            CartesianGrid::new(e, 10, 0),
            Err(invalid_geometry("height", "must be greater than zero"))
        );
        assert!(CartesianGrid::new(e, usize::MAX, 2).is_err());
    }

    #[test]
    fn test_grid_rejects_malformed_extent() {
        let reject = |e: GeoExtent, parameter: &'static str| {
            assert!(matches!(
                CartesianGrid::new(e, 10, 10),
                Err(Error::InvalidGeometry { parameter: p, .. }) if p == parameter
            ));
        };
        reject(extent((36.0, -99.0), (34.0, -97.0)), "extent.latitude");
        reject(extent((34.0, -97.0), (36.0, -99.0)), "extent.longitude");
        reject(extent((f64::NAN, -99.0), (36.0, -97.0)), "extent.latitude");
        reject(extent((34.0, -99.0), (91.0, -97.0)), "extent.latitude");
        reject(extent((34.0, -181.0), (36.0, -97.0)), "extent.longitude");
    }
}
