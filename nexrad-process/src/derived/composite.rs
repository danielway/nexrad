use crate::result::{Error, Result};
use crate::VolumeDerivedProduct;
use nexrad_model::data::{CartesianField, GateStatus, Scan, SweepField};
use nexrad_model::geo::{GeoExtent, GeoPoint, RadarCoordinateSystem};

/// Composite reflectivity (CREF) — the maximum reflectivity at each geographic
/// point across all elevation tilts.
///
/// This is one of the most commonly used derived products in operational
/// meteorology. It provides a plan view of the strongest echoes in the volume
/// regardless of altitude, making it useful for identifying convective storms.
///
/// # Algorithm
///
/// For each cell in the output geographic grid:
/// 1. Convert the cell center to a polar coordinate for each elevation tilt
/// 2. Sample the reflectivity value at that polar coordinate from each sweep field
/// 3. Take the maximum valid value across all tilts
///
/// # Example
///
/// ```ignore
/// use nexrad_process::derived::CompositeReflectivity;
/// use nexrad_process::VolumeDerivedProduct;
///
/// let cref = CompositeReflectivity;
/// let field = cref.compute(&scan, &ref_fields, &coord_sys, &extent, (800, 800))?;
/// ```
pub struct CompositeReflectivity;

impl VolumeDerivedProduct for CompositeReflectivity {
    fn name(&self) -> &str {
        "CompositeReflectivity"
    }

    fn compute(
        &self,
        _scan: &Scan,
        fields: &[SweepField],
        coord_system: &RadarCoordinateSystem,
        output_extent: &GeoExtent,
        output_resolution: (usize, usize),
    ) -> Result<CartesianField> {
        if fields.is_empty() {
            return Err(Error::MissingData("no sweep fields provided".to_string()));
        }

        let (width, height) = output_resolution;
        if width == 0 || height == 0 {
            return Err(Error::InvalidParameter(
                "output resolution must be > 0".to_string(),
            ));
        }

        let unit = fields[0].unit().to_string();
        let mut output = CartesianField::new(
            "Composite Reflectivity",
            &unit,
            *output_extent,
            width,
            height,
        );

        let lat_range = output_extent.max.latitude - output_extent.min.latitude;
        let lon_range = output_extent.max.longitude - output_extent.min.longitude;

        for row in 0..height {
            // Row 0 = north edge (max latitude)
            let lat = output_extent.max.latitude - (row as f64 + 0.5) / height as f64 * lat_range;

            for col in 0..width {
                let lon =
                    output_extent.min.longitude + (col as f64 + 0.5) / width as f64 * lon_range;

                let geo_point = GeoPoint {
                    latitude: lat,
                    longitude: lon,
                };

                let mut max_value = f32::MIN;
                let mut found_valid = false;

                for field in fields {
                    let polar = coord_system.geo_to_polar(geo_point, field.elevation_degrees());

                    if let Some((val, status)) =
                        field.value_at_polar(polar.azimuth_degrees, polar.range_km)
                    {
                        if status == GateStatus::Valid && val > max_value {
                            max_value = val;
                            found_valid = true;
                        }
                    }
                }

                if found_valid {
                    output.set(row, col, max_value, GateStatus::Valid);
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::data::{PulseWidth, Scan, SweepField, VolumeCoveragePattern};
    use nexrad_model::geo::RadarCoordinateSystem;
    use nexrad_model::meta::Site;

    fn test_site() -> Site {
        Site::new(*b"KTLX", 35.3331, -97.2778, 370, 10)
    }

    fn test_scan() -> Scan {
        let vcp = VolumeCoveragePattern::new(
            215,
            1,
            0.5,
            PulseWidth::Short,
            false,
            0,
            false,
            0,
            false,
            false,
            0,
            false,
            false,
            vec![],
        );
        Scan::new(vcp, vec![])
    }

    fn make_uniform_field(elevation: f32, value: f32) -> SweepField {
        let mut azimuths = Vec::new();
        for i in 0..360 {
            azimuths.push(i as f32);
        }

        let gate_count = 100;
        let mut field = SweepField::new_empty(
            "Reflectivity",
            "dBZ",
            elevation,
            azimuths,
            1.0,
            2.0,
            0.25,
            gate_count,
        );

        for az in 0..360 {
            for gate in 0..gate_count {
                field.set(az, gate, value, GateStatus::Valid);
            }
        }

        field
    }

    #[test]
    fn test_cref_single_tilt() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let extent = coord_sys.sweep_extent(25.0);
        let scan = test_scan();

        let fields = vec![make_uniform_field(0.5, 30.0)];

        let result = CompositeReflectivity
            .compute(&scan, &fields, &coord_sys, &extent, (10, 10))
            .unwrap();

        // Center pixel should have data
        let (val, status) = result.get(5, 5);
        assert_eq!(status, GateStatus::Valid);
        assert_eq!(val, 30.0);
    }

    #[test]
    fn test_cref_takes_max() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let extent = coord_sys.sweep_extent(25.0);
        let scan = test_scan();

        let fields = vec![
            make_uniform_field(0.5, 20.0),
            make_uniform_field(1.5, 40.0),
            make_uniform_field(3.5, 25.0),
        ];

        let result = CompositeReflectivity
            .compute(&scan, &fields, &coord_sys, &extent, (10, 10))
            .unwrap();

        // Center pixel should have the max value (40.0)
        let (val, status) = result.get(5, 5);
        assert_eq!(status, GateStatus::Valid);
        assert_eq!(val, 40.0);
    }

    #[test]
    fn test_cref_empty_fields_error() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let extent = coord_sys.sweep_extent(25.0);
        let scan = test_scan();

        let result = CompositeReflectivity.compute(&scan, &[], &coord_sys, &extent, (10, 10));
        assert!(result.is_err());
    }
}
