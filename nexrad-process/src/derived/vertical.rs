use crate::result::{Error, Result};
use nexrad_model::data::{GateStatus, SweepField, VerticalField};

/// Effective earth radius in km using the standard 4/3 refraction model.
const EFFECTIVE_EARTH_RADIUS_KM: f64 = 6371.0 * 4.0 / 3.0;

/// Vertical cross-section (RHI-style) — assembles a range-height display from
/// PPI scan data at a fixed azimuth.
///
/// For each cell in the output grid, the beam-height equation (standard 4/3
/// earth-radius model) is used to map range and altitude back to each
/// elevation tilt's polar coordinates. The maximum valid value across all
/// tilts is retained, producing a pseudo-RHI from PPI scan data.
///
/// Unlike [`CompositeReflectivity`](super::CompositeReflectivity), this type does not
/// implement [`ScanDerivedProduct`](crate::ScanDerivedProduct) because its output is a
/// [`VerticalField`] (range vs. altitude) rather than a
/// [`CartesianField`](nexrad_model::data::CartesianField). Its
/// construction parameters (azimuth, range, altitude, dimensions) fully define the
/// output grid, so no coordinate system or geographic extent is needed at compute time.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::derived::VerticalCrossSection;
/// use nexrad_model::data::{SweepField, Product};
///
/// // Extract reflectivity fields from all sweeps
/// let ref_fields: Vec<SweepField> = scan.sweeps().iter()
///     .filter_map(|s| SweepField::from_radials(s.radials(), Product::Reflectivity))
///     .collect();
///
/// // Create a cross-section at 200° azimuth, 0-230 km range, 0-18 km altitude
/// let vcs = VerticalCrossSection::new(200.0, 230.0, 18000.0, 600, 300)?;
/// let vertical_field = vcs.compute(&ref_fields)?;
///
/// // Render with nexrad-render
/// use nexrad_render::{render_vertical, default_scale, RenderOptions};
/// let scale = default_scale(Product::Reflectivity);
/// let result = render_vertical(&vertical_field, &scale, &RenderOptions::new(1200, 600))?;
/// result.save("vertical_cross_section.png")?;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct VerticalCrossSection {
    /// Target azimuth in degrees (0-360).
    azimuth_angle_degrees: f64,
    /// Maximum range in km for the horizontal axis.
    max_range_km: f64,
    /// Maximum altitude in meters for the vertical axis.
    max_altitude_meters: f64,
    /// Output grid width (range bins).
    width: usize,
    /// Output grid height (altitude bins).
    height: usize,
}

impl VerticalCrossSection {
    /// Create a new vertical cross-section builder.
    ///
    /// # Parameters
    ///
    /// - `azimuth_angle_degrees` — target azimuth (0-360 degrees)
    /// - `max_range_km` — horizontal extent in km
    /// - `max_altitude_meters` — vertical extent in meters
    /// - `width` — number of horizontal bins (columns)
    /// - `height` — number of vertical bins (rows)
    ///
    /// # Errors
    ///
    /// Returns an error if any parameter is non-positive.
    pub fn new(
        azimuth_angle_degrees: f64,
        max_range_km: f64,
        max_altitude_meters: f64,
        width: usize,
        height: usize,
    ) -> Result<Self> {
        if max_range_km <= 0.0 {
            return Err(Error::InvalidParameter(
                "max_range_km must be positive".to_string(),
            ));
        }
        if max_altitude_meters <= 0.0 {
            return Err(Error::InvalidParameter(
                "max_altitude_meters must be positive".to_string(),
            ));
        }
        if width == 0 || height == 0 {
            return Err(Error::InvalidParameter(
                "output dimensions must be > 0".to_string(),
            ));
        }
        Ok(Self {
            azimuth_angle_degrees,
            max_range_km,
            max_altitude_meters,
            width,
            height,
        })
    }

    /// Compute the vertical cross-section from sweep fields at multiple elevations.
    ///
    /// Each input field contributes data at the altitude determined by its elevation
    /// angle and the beam-height equation. Where multiple tilts overlap the same
    /// output cell, the maximum valid value is retained. The label and unit of the
    /// output field are taken from the first input field.
    ///
    /// # Errors
    ///
    /// Returns an error if no fields are provided.
    pub fn compute(&self, fields: &[SweepField]) -> Result<VerticalField> {
        if fields.is_empty() {
            return Err(Error::MissingData("no sweep fields provided".to_string()));
        }

        let label = format!("{} RHI", fields[0].label());
        let unit = fields[0].unit().to_string();

        let mut output = VerticalField::new(
            label,
            unit,
            (0.0, self.max_range_km),
            (0.0, self.max_altitude_meters),
            self.width,
            self.height,
        )?;

        let re = EFFECTIVE_EARTH_RADIUS_KM;

        for field in fields {
            let elev_rad = (field.elevation_angle_degrees() as f64).to_radians();

            for col in 0..self.width {
                let range_km = (col as f64 + 0.5) / self.width as f64 * self.max_range_km;

                // Beam height using 4/3 earth radius model
                let altitude_km = range_km * elev_rad.sin() + (range_km * range_km) / (2.0 * re);
                let altitude_m = altitude_km * 1000.0;

                if altitude_m < 0.0 || altitude_m > self.max_altitude_meters {
                    continue;
                }

                // Row 0 = top = highest altitude
                let row = ((self.max_altitude_meters - altitude_m) / self.max_altitude_meters
                    * self.height as f64) as usize;
                if row >= self.height {
                    continue;
                }

                if let Some((val, status)) =
                    field.value_at_polar(self.azimuth_angle_degrees, range_km)
                {
                    if status == GateStatus::Valid {
                        let Some((existing_val, existing_status)) = output.get(row, col) else {
                            continue;
                        };
                        if existing_status != GateStatus::Valid || val > existing_val {
                            output.set(row, col, val, GateStatus::Valid)?;
                        }
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_uniform_field(elevation: f32, value: f32) -> SweepField {
        let mut azimuths = Vec::new();
        for i in 0..360 {
            azimuths.push(i as f32);
        }

        let gate_count = 100;
        let mut field = SweepField::builder("Reflectivity", "dBZ")
            .elevation_angle_degrees(elevation)
            .azimuth_angles_degrees(azimuths)
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(gate_count)
            .build()
            .expect("valid geometry");

        for az in 0..360 {
            for gate in 0..gate_count {
                field.set(az, gate, value, GateStatus::Valid).unwrap();
            }
        }

        field
    }

    #[test]
    fn test_vertical_basic() {
        let fields = vec![make_uniform_field(0.5, 30.0), make_uniform_field(3.5, 40.0)];

        let vcs = VerticalCrossSection::new(180.0, 25.0, 5000.0, 50, 25).unwrap();
        let result = vcs.compute(&fields).unwrap();

        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 25);

        // Should have some valid data (low-elevation beam at close range)
        let mut found_valid = false;
        for row in 0..result.height() {
            for col in 0..result.width() {
                let (_, status) = result.get(row, col).unwrap();
                if status == GateStatus::Valid {
                    found_valid = true;
                }
            }
        }
        assert!(found_valid, "expected at least some valid data");
    }

    #[test]
    fn test_vertical_takes_max() {
        let fields = vec![make_uniform_field(0.5, 20.0), make_uniform_field(0.5, 40.0)];

        let vcs = VerticalCrossSection::new(180.0, 25.0, 5000.0, 50, 25).unwrap();
        let result = vcs.compute(&fields).unwrap();

        // Overlapping beams should take the max value
        for row in 0..result.height() {
            for col in 0..result.width() {
                let (val, status) = result.get(row, col).unwrap();
                if status == GateStatus::Valid {
                    assert_eq!(val, 40.0);
                }
            }
        }
    }

    #[test]
    fn test_vertical_skips_filtered_gates() {
        let mut filtered = make_uniform_field(0.5, 60.0);
        for az in 0..filtered.azimuth_count() {
            for gate in 0..filtered.gate_count() {
                filtered.set(az, gate, 60.0, GateStatus::Filtered).unwrap();
            }
        }
        let vcs = VerticalCrossSection::new(180.0, 25.0, 5000.0, 50, 25).unwrap();

        // Alongside a valid tilt, only the valid values appear.
        let result = vcs
            .compute(&[make_uniform_field(0.5, 20.0), filtered.clone()])
            .unwrap();
        let mut found_valid = false;
        for row in 0..result.height() {
            for col in 0..result.width() {
                let (val, status) = result.get(row, col).unwrap();
                assert_ne!(status, GateStatus::Filtered);
                if status == GateStatus::Valid {
                    assert_eq!(val, 20.0);
                    found_valid = true;
                }
            }
        }
        assert!(found_valid, "expected at least some valid data");

        // A fully filtered tilt contributes nothing.
        let result = vcs.compute(&[filtered]).unwrap();
        assert!(result
            .statuses()
            .iter()
            .all(|status| *status == GateStatus::NoData));
    }

    #[test]
    fn test_vertical_empty_fields_error() {
        let vcs = VerticalCrossSection::new(180.0, 25.0, 5000.0, 50, 25).unwrap();
        assert!(vcs.compute(&[]).is_err());
    }

    #[test]
    fn test_vertical_invalid_params() {
        assert!(VerticalCrossSection::new(180.0, 0.0, 5000.0, 50, 25).is_err());
        assert!(VerticalCrossSection::new(180.0, 25.0, 0.0, 50, 25).is_err());
        assert!(VerticalCrossSection::new(180.0, 25.0, 5000.0, 0, 25).is_err());
        assert!(VerticalCrossSection::new(180.0, 25.0, 5000.0, 50, 0).is_err());
    }
}
