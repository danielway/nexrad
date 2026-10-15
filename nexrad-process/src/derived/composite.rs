use crate::result::{check_product, Error, Result};
use crate::{CartesianGrid, ScanDerivedProduct};
use nexrad_model::data::{CartesianField, GateStatus, Product, SweepField};
use nexrad_model::geo::{GeoPoint, RadarCoordinateSystem};

/// Composite reflectivity (CREF) — the maximum reflectivity at each geographic
/// point across all elevation tilts.
///
/// This is one of the most commonly used derived products in operational
/// meteorology. It provides a plan view of the strongest echoes in the scan
/// regardless of altitude, making it useful for identifying convective storms.
///
/// # Algorithm
///
/// For each cell in the output geographic grid:
/// 1. Convert the cell center to a polar coordinate for each elevation tilt
/// 2. Sample the reflectivity value at that polar coordinate from each sweep field
/// 3. Take the maximum valid value across all tilts
///
/// Every input field must hold [`Product::Reflectivity`]. The output field is labelled
/// "Composite Reflectivity" with unit "dBZ".
///
/// # Example
///
/// ```
/// use nexrad_process::derived::CompositeReflectivity;
/// use nexrad_process::{CartesianGrid, ScanDerivedProduct};
/// # let site = nexrad_model::meta::Site::builder(*b"KTLX", 35.3331, -97.2778).build()?;
/// # let coord_sys = nexrad_model::geo::RadarCoordinateSystem::new(&site);
/// # let lowest_tilt = nexrad_model::data::SweepField::builder("Reflectivity", "dBZ")
/// #     .product(nexrad_model::data::Product::Reflectivity)
/// #     .elevation_angle_degrees(0.5)
/// #     .azimuth_angles_degrees((0..360).map(|a| a as f32).collect())
/// #     .first_gate_range_km(2.0)
/// #     .gate_interval_km(0.25)
/// #     .gate_count(400)
/// #     .build()?;
/// # let ref_fields = vec![lowest_tilt];
///
/// let grid = CartesianGrid::new(coord_sys.sweep_extent(230.0), 800, 800)?;
/// let field = CompositeReflectivity.compute(&ref_fields, &coord_sys, &grid)?;
/// # Ok::<(), nexrad_process::Error>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CompositeReflectivity;

impl ScanDerivedProduct for CompositeReflectivity {
    fn name(&self) -> &str {
        "CompositeReflectivity"
    }

    /// # Errors
    ///
    /// Returns [`Error::MissingData`] if `fields` is empty and
    /// [`Error::UnexpectedProduct`] if any field does not hold [`Product::Reflectivity`].
    fn compute(
        &self,
        fields: &[SweepField],
        coordinate_system: &RadarCoordinateSystem,
        grid: &CartesianGrid,
    ) -> Result<CartesianField> {
        if fields.is_empty() {
            return Err(Error::MissingData {
                what: "no sweep fields provided",
            });
        }
        for field in fields {
            check_product(field, Product::Reflectivity)?;
        }

        let output_extent = grid.extent();
        let (width, height) = (grid.width(), grid.height());
        let mut output = CartesianField::new(
            "Composite Reflectivity",
            Product::Reflectivity.unit(),
            *output_extent,
            width,
            height,
        )?;

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
                    let polar = coordinate_system
                        .geo_to_polar(geo_point, field.elevation_angle_degrees() as f64);

                    if let Some((val, status)) =
                        field.value_at_polar(polar.azimuth_angle_degrees, polar.range_km)
                    {
                        if status == GateStatus::Valid && val > max_value {
                            max_value = val;
                            found_valid = true;
                        }
                    }
                }

                if found_valid {
                    output.set(row, col, max_value, GateStatus::Valid)?;
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::data::SweepField;
    use nexrad_model::geo::RadarCoordinateSystem;
    use nexrad_model::meta::Site;

    fn test_site() -> Site {
        Site::builder(*b"KTLX", 35.3331, -97.2778)
            .site_elevation_meters(370)
            .feedhorn_height_meters(10)
            .build()
            .unwrap()
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
    fn test_cref_single_tilt() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let grid = CartesianGrid::new(coord_sys.sweep_extent(25.0), 10, 10).unwrap();

        let fields = vec![make_uniform_field(0.5, 30.0)];

        let result = CompositeReflectivity
            .compute(&fields, &coord_sys, &grid)
            .unwrap();

        // Center pixel should have data
        let (val, status) = result.get(5, 5).unwrap();
        assert_eq!(status, GateStatus::Valid);
        assert_eq!(val, 30.0);
    }

    #[test]
    fn test_cref_takes_max() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let grid = CartesianGrid::new(coord_sys.sweep_extent(25.0), 10, 10).unwrap();

        let fields = vec![
            make_uniform_field(0.5, 20.0),
            make_uniform_field(1.5, 40.0),
            make_uniform_field(3.5, 25.0),
        ];

        let result = CompositeReflectivity
            .compute(&fields, &coord_sys, &grid)
            .unwrap();

        // Center pixel should have the max value (40.0)
        let (val, status) = result.get(5, 5).unwrap();
        assert_eq!(status, GateStatus::Valid);
        assert_eq!(val, 40.0);
    }

    #[test]
    fn test_cref_skips_filtered_gates() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let grid = CartesianGrid::new(coord_sys.sweep_extent(25.0), 10, 10).unwrap();

        let mut filtered = make_uniform_field(1.5, 60.0);
        for az in 0..filtered.azimuth_count() {
            for gate in 0..filtered.gate_count() {
                filtered.set(az, gate, 60.0, GateStatus::Filtered).unwrap();
            }
        }

        let result = CompositeReflectivity
            .compute(
                &[make_uniform_field(0.5, 20.0), filtered.clone()],
                &coord_sys,
                &grid,
            )
            .unwrap();
        assert_eq!(result.get(5, 5), Some((20.0, GateStatus::Valid)));

        let result = CompositeReflectivity
            .compute(&[filtered], &coord_sys, &grid)
            .unwrap();
        assert_eq!(
            result.get(5, 5).map(|(_, status)| status),
            Some(GateStatus::NoData)
        );
    }

    #[test]
    fn test_cref_empty_fields_error() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let grid = CartesianGrid::new(coord_sys.sweep_extent(25.0), 10, 10).unwrap();

        let result = CompositeReflectivity.compute(&[], &coord_sys, &grid);
        assert!(result.is_err());
    }

    #[test]
    fn test_cref_output_label_and_grid() -> Result<()> {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let grid = CartesianGrid::new(coord_sys.sweep_extent(25.0), 12, 8)?;

        let result =
            CompositeReflectivity.compute(&[make_uniform_field(0.5, 30.0)], &coord_sys, &grid)?;
        assert_eq!(result.label(), "Composite Reflectivity");
        assert_eq!(result.unit(), "dBZ");
        assert_eq!(result.extent(), grid.extent());
        assert_eq!((result.width(), result.height()), (12, 8));
        Ok(())
    }

    #[test]
    fn test_cref_rejects_non_reflectivity_fields() {
        let coord_sys = RadarCoordinateSystem::new(&test_site());
        let grid = CartesianGrid::new(coord_sys.sweep_extent(25.0), 10, 10).unwrap();

        let unlabelled = SweepField::builder("Reflectivity", "dBZ")
            .azimuth_angles_degrees(vec![0.0, 1.0])
            .gate_interval_km(0.25)
            .gate_count(4)
            .build()
            .unwrap();
        assert_eq!(
            CompositeReflectivity.compute(
                &[make_uniform_field(0.5, 30.0), unlabelled],
                &coord_sys,
                &grid
            ),
            Err(Error::UnexpectedProduct {
                expected: Product::Reflectivity,
                actual: None,
            })
        );

        let velocity = SweepField::builder("Velocity", "m/s")
            .product(Product::Velocity)
            .azimuth_angles_degrees(vec![0.0, 1.0])
            .gate_interval_km(0.25)
            .gate_count(4)
            .build()
            .unwrap();
        assert_eq!(
            CompositeReflectivity.compute(&[velocity], &coord_sys, &grid),
            Err(Error::UnexpectedProduct {
                expected: Product::Reflectivity,
                actual: Some(Product::Velocity),
            })
        );
    }
}
