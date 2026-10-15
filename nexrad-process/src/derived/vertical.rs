use crate::result::{check_product, Error, Result};
use nexrad_model::data::{GateStatus, Product, SweepField, VerticalField};
use nexrad_model::geo::RadarCoordinateSystem;

/// Vertical cross-section (RHI-style) — assembles a range-height display from
/// PPI scan data at a fixed azimuth.
///
/// Each elevation tilt is sampled along the target azimuth and placed at its ground range and
/// beam height above mean sea level from
/// [`RadarCoordinateSystem::ground_range_km`] and [`RadarCoordinateSystem::beam_height_meters`]
/// (spherical earth with 4/3 effective-radius refraction, including the antenna height). The
/// maximum valid value across all tilts is retained, producing a pseudo-RHI from PPI scan data.
/// The horizontal axis is ground range in kilometers and the vertical axis is altitude above
/// mean sea level in meters.
///
/// Unlike [`CompositeReflectivity`](super::CompositeReflectivity), this type does not
/// implement [`ScanDerivedProduct`](crate::ScanDerivedProduct) because its output is a
/// [`VerticalField`] (range vs. altitude) rather than a
/// [`CartesianField`](nexrad_model::data::CartesianField). Its
/// construction parameters (azimuth, range, altitude, dimensions) fully define the
/// output grid; the radar's coordinate system supplies the beam geometry at compute time.
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
/// let vcs = VerticalCrossSection::builder(200.0)
///     .max_range_km(230.0)
///     .max_altitude_meters(18000.0)
///     .width(600)
///     .height(300)
///     .build()?;
/// let coord_sys = scan.coordinate_system().unwrap();
/// let vertical_field = vcs.compute(&ref_fields, &coord_sys)?;
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
    /// Start building a cross-section along `azimuth_angle_degrees`, in degrees clockwise
    /// from north.
    ///
    /// The other parameters default to those of [`VerticalCrossSectionBuilder`].
    pub fn builder(azimuth_angle_degrees: f64) -> VerticalCrossSectionBuilder {
        VerticalCrossSectionBuilder {
            azimuth_angle_degrees,
            max_range_km: 230.0,
            max_altitude_meters: 18_000.0,
            width: 600,
            height: 300,
        }
    }

    /// Target azimuth in degrees clockwise from north.
    pub fn azimuth_angle_degrees(&self) -> f64 {
        self.azimuth_angle_degrees
    }

    /// Horizontal extent in km of ground range.
    pub fn max_range_km(&self) -> f64 {
        self.max_range_km
    }

    /// Vertical extent in meters above mean sea level.
    pub fn max_altitude_meters(&self) -> f64 {
        self.max_altitude_meters
    }

    /// Number of horizontal bins (columns) in the output.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of vertical bins (rows) in the output.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Compute the vertical cross-section from sweep fields at multiple elevations.
    ///
    /// Each input field contributes data at the ground range and altitude that
    /// `coordinate_system` gives for its elevation angle. Where multiple tilts overlap the same
    /// output cell, the maximum valid value is retained. The label and unit of the
    /// output field are taken from the first input field.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingData`] if no fields are provided and
    /// [`Error::UnexpectedProduct`] if any field does not hold [`Product::Reflectivity`].
    pub fn compute(
        &self,
        fields: &[SweepField],
        coordinate_system: &RadarCoordinateSystem,
    ) -> Result<VerticalField> {
        if fields.is_empty() {
            return Err(Error::MissingData {
                what: "no sweep fields provided",
            });
        }
        for field in fields {
            check_product(field, Product::Reflectivity)?;
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

        for field in fields {
            let elevation_angle_degrees = field.elevation_angle_degrees() as f64;

            for sample in 0..self.width {
                let range_km = (sample as f64 + 0.5) / self.width as f64 * self.max_range_km;

                let altitude_m =
                    coordinate_system.beam_height_meters(range_km, elevation_angle_degrees);
                if altitude_m < 0.0 || altitude_m > self.max_altitude_meters {
                    continue;
                }

                let ground_range_km =
                    coordinate_system.ground_range_km(range_km, elevation_angle_degrees);
                let col = (ground_range_km / self.max_range_km * self.width as f64) as usize;
                if col >= self.width {
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

/// Builder for a [`VerticalCrossSection`], created by [`VerticalCrossSection::builder`].
///
/// Defaults: a 230 km ground range, an 18 000 m altitude, and a 600 x 300 output grid.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct VerticalCrossSectionBuilder {
    azimuth_angle_degrees: f64,
    max_range_km: f64,
    max_altitude_meters: f64,
    width: usize,
    height: usize,
}

impl VerticalCrossSectionBuilder {
    /// Set the horizontal extent in km of ground range; must be finite and positive.
    pub fn max_range_km(mut self, max_range_km: f64) -> Self {
        self.max_range_km = max_range_km;
        self
    }

    /// Set the vertical extent in meters above mean sea level; must be finite and positive.
    pub fn max_altitude_meters(mut self, max_altitude_meters: f64) -> Self {
        self.max_altitude_meters = max_altitude_meters;
        self
    }

    /// Set the number of horizontal bins (columns); must be greater than zero.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the number of vertical bins (rows); must be greater than zero.
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Build the [`VerticalCrossSection`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if the azimuth is not finite or outside
    /// `0.0..360.0`, either extent is not finite and positive, or either dimension is zero.
    pub fn build(self) -> Result<VerticalCrossSection> {
        if !(self.azimuth_angle_degrees.is_finite()
            && (0.0..360.0).contains(&self.azimuth_angle_degrees))
        {
            return Err(Error::InvalidParameter {
                parameter: "azimuth_angle_degrees",
                reason: "must be finite and in 0.0..360.0",
            });
        }
        let positive = |parameter: &'static str, value: f64| {
            if value.is_finite() && value > 0.0 {
                Ok(())
            } else {
                Err(Error::InvalidParameter {
                    parameter,
                    reason: "must be finite and greater than zero",
                })
            }
        };
        positive("max_range_km", self.max_range_km)?;
        positive("max_altitude_meters", self.max_altitude_meters)?;
        for (parameter, value) in [("width", self.width), ("height", self.height)] {
            if value == 0 {
                return Err(Error::InvalidParameter {
                    parameter,
                    reason: "must be greater than zero",
                });
            }
        }
        Ok(VerticalCrossSection {
            azimuth_angle_degrees: self.azimuth_angle_degrees,
            max_range_km: self.max_range_km,
            max_altitude_meters: self.max_altitude_meters,
            width: self.width,
            height: self.height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::meta::Site;

    fn section(
        azimuth: f64,
        max_range_km: f64,
        max_altitude_meters: f64,
        width: usize,
        height: usize,
    ) -> Result<VerticalCrossSection> {
        VerticalCrossSection::builder(azimuth)
            .max_range_km(max_range_km)
            .max_altitude_meters(max_altitude_meters)
            .width(width)
            .height(height)
            .build()
    }

    fn coordinate_system() -> RadarCoordinateSystem {
        let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
            .build()
            .expect("valid site");
        RadarCoordinateSystem::new(&site)
    }

    fn make_uniform_field(elevation: f32, value: f32) -> SweepField {
        let mut azimuths = Vec::new();
        for i in 0..360 {
            azimuths.push(i as f32);
        }

        let gate_count = 100;
        let mut field = SweepField::builder("Reflectivity", "dBZ")
            .product(Product::Reflectivity)
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

        let vcs = section(180.0, 25.0, 5000.0, 50, 25).unwrap();
        let result = vcs.compute(&fields, &coordinate_system()).unwrap();

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

        let vcs = section(180.0, 25.0, 5000.0, 50, 25).unwrap();
        let result = vcs.compute(&fields, &coordinate_system()).unwrap();

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
        let vcs = section(180.0, 25.0, 5000.0, 50, 25).unwrap();

        // Alongside a valid tilt, only the valid values appear.
        let result = vcs
            .compute(
                &[make_uniform_field(0.5, 20.0), filtered.clone()],
                &coordinate_system(),
            )
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
        let result = vcs.compute(&[filtered], &coordinate_system()).unwrap();
        assert!(result
            .statuses()
            .iter()
            .all(|status| *status == GateStatus::NoData));
    }

    #[test]
    fn test_vertical_empty_fields_error() {
        let vcs = section(180.0, 25.0, 5000.0, 50, 25).unwrap();
        assert!(vcs.compute(&[], &coordinate_system()).is_err());
    }

    #[test]
    fn test_vertical_requires_reflectivity() {
        let vcs = section(180.0, 25.0, 5000.0, 50, 25).unwrap();
        let velocity = SweepField::builder("Velocity", "m/s")
            .product(Product::Velocity)
            .azimuth_angles_degrees(vec![0.0, 1.0])
            .gate_interval_km(0.25)
            .gate_count(4)
            .build()
            .unwrap();
        assert_eq!(
            vcs.compute(
                &[make_uniform_field(0.5, 30.0), velocity],
                &coordinate_system()
            ),
            Err(Error::UnexpectedProduct {
                expected: Product::Reflectivity,
                actual: Some(Product::Velocity),
            })
        );
    }

    #[test]
    fn test_vertical_builder_and_getters() -> Result<()> {
        let vcs = section(200.0, 100.0, 12_000.0, 40, 20)?;
        assert_eq!(vcs.azimuth_angle_degrees(), 200.0);
        assert_eq!(vcs.max_range_km(), 100.0);
        assert_eq!(vcs.max_altitude_meters(), 12_000.0);
        assert_eq!(vcs.width(), 40);
        assert_eq!(vcs.height(), 20);

        let defaults = VerticalCrossSection::builder(0.0).build()?;
        assert_eq!(defaults.max_range_km(), 230.0);
        assert_eq!(defaults.max_altitude_meters(), 18_000.0);
        assert_eq!((defaults.width(), defaults.height()), (600, 300));
        Ok(())
    }

    #[test]
    fn test_vertical_invalid_params() {
        let rejected = |result: Result<VerticalCrossSection>| match result {
            Err(Error::InvalidParameter { parameter, .. }) => parameter,
            other => panic!("expected InvalidParameter, got {other:?}"),
        };
        assert_eq!(
            rejected(section(f64::NAN, 25.0, 5000.0, 50, 25)),
            "azimuth_angle_degrees"
        );
        assert_eq!(
            rejected(section(360.0, 25.0, 5000.0, 50, 25)),
            "azimuth_angle_degrees"
        );
        assert_eq!(
            rejected(section(-0.5, 25.0, 5000.0, 50, 25)),
            "azimuth_angle_degrees"
        );
        assert_eq!(
            rejected(section(180.0, 0.0, 5000.0, 50, 25)),
            "max_range_km"
        );
        assert_eq!(
            rejected(section(180.0, f64::INFINITY, 5000.0, 50, 25)),
            "max_range_km"
        );
        assert_eq!(
            rejected(section(180.0, 25.0, 0.0, 50, 25)),
            "max_altitude_meters"
        );
        assert_eq!(
            rejected(section(180.0, 25.0, f64::NAN, 50, 25)),
            "max_altitude_meters"
        );
        assert_eq!(rejected(section(180.0, 25.0, 5000.0, 0, 25)), "width");
        assert_eq!(rejected(section(180.0, 25.0, 5000.0, 50, 0)), "height");
    }
}
