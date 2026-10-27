use crate::result::{check_finite, check_product, Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, Product, SweepField};

/// Storm-relative velocity — subtracts a storm motion vector from radial velocity.
///
/// The input field must hold [`Product::Velocity`]. The output field has the same geometry,
/// is labelled "Storm-Relative Velocity" with unit "m/s", and has no
/// [`product`](SweepField::product), since it is derived rather than a Level II moment.
///
/// The storm motion is specified as a direction and a speed (m/s). The direction is the
/// direction the storm moves **from**, in degrees clockwise from north, so `180.0` is a
/// storm moving from the south toward the north. The processor turns it into a motion vector
/// pointing toward `direction + 180°`, projects that vector onto each radial (the unit vector
/// from the radar along the radial's azimuth) and subtracts the projection from every valid
/// gate on the radial. Gates with any other status keep their value and status.
///
/// With positive input velocities meaning motion away from the radar, the sign consequence is:
///
/// - on the radial whose azimuth equals `direction + 180°` (the way the storm is heading),
///   `storm_speed_mps` is subtracted;
/// - on the radial whose azimuth equals `direction` (where the storm is coming from),
///   `storm_speed_mps` is added;
/// - on radials at right angles to the motion, values are unchanged.
///
/// Passing the direction the storm moves *toward* instead reverses the sign of the correction.
///
/// # Example
///
/// ```
/// use nexrad_process::derived::StormRelativeVelocity;
/// use nexrad_process::SweepProcessor;
/// # let velocity_field = nexrad_model::data::SweepField::builder("Velocity", "m/s")
/// #     .product(nexrad_model::data::Product::Velocity)
/// #     .elevation_angle_degrees(0.5)
/// #     .azimuth_angles_degrees((0..360).map(|a| a as f32).collect())
/// #     .first_gate_range_km(2.0)
/// #     .gate_interval_km(0.25)
/// #     .gate_count(400)
/// #     .build()?;
///
/// // Storm moving from 240° at 15 m/s
/// let srv = StormRelativeVelocity::new(240.0, 15.0)?;
/// let sr_velocity = srv.process(&velocity_field)?;
/// # Ok::<(), nexrad_process::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StormRelativeVelocity {
    /// Storm motion direction in degrees clockwise from north (0-360).
    storm_direction_degrees: f32,
    /// Storm motion speed in m/s.
    storm_speed_mps: f32,
}

impl StormRelativeVelocity {
    /// Create a new storm-relative velocity processor.
    ///
    /// # Parameters
    ///
    /// - `storm_direction_degrees` — Direction the storm is moving from (not toward), in
    ///   degrees clockwise from north, in `0.0..360.0`.
    /// - `storm_speed_mps` — Storm movement speed in m/s.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if the direction is not finite or outside
    /// `0.0..360.0`, or the speed is not finite or is negative.
    pub fn new(storm_direction_degrees: f32, storm_speed_mps: f32) -> Result<Self> {
        check_finite("storm_direction_degrees", storm_direction_degrees)?;
        if !(0.0..360.0).contains(&storm_direction_degrees) {
            return Err(Error::InvalidParameter {
                parameter: "storm_direction_degrees",
                reason: "must be in 0.0..360.0",
            });
        }
        check_finite("storm_speed_mps", storm_speed_mps)?;
        if storm_speed_mps < 0.0 {
            return Err(Error::InvalidParameter {
                parameter: "storm_speed_mps",
                reason: "must not be negative",
            });
        }
        Ok(Self {
            storm_direction_degrees,
            storm_speed_mps,
        })
    }

    /// Direction the storm is moving from (not toward), in degrees clockwise from north.
    pub fn storm_direction_degrees(&self) -> f32 {
        self.storm_direction_degrees
    }

    /// Storm movement speed in m/s.
    pub fn storm_speed_mps(&self) -> f32 {
        self.storm_speed_mps
    }
}

impl SweepProcessor for StormRelativeVelocity {
    fn name(&self) -> &str {
        "StormRelativeVelocity"
    }

    /// # Errors
    ///
    /// Returns [`Error::UnexpectedProduct`] if `input` does not hold [`Product::Velocity`].
    fn process(&self, input: &SweepField) -> Result<SweepField> {
        check_product(input, Product::Velocity)?;

        let mut output = SweepField::builder("Storm-Relative Velocity", Product::Velocity.unit())
            .elevation_angle_degrees(input.elevation_angle_degrees())
            .azimuth_angles_degrees(input.azimuth_angles_degrees().to_vec())
            .azimuth_spacing_degrees(input.azimuth_spacing_degrees())
            .first_gate_range_km(input.first_gate_range_km())
            .gate_interval_km(input.gate_interval_km())
            .gate_count(input.gate_count())
            .build()?;

        // Convert storm direction to radians (meteorological convention: FROM direction)
        // The storm motion vector points in the direction the storm is moving TO.
        let storm_to_rad = (self.storm_direction_degrees + 180.0).to_radians();

        // Storm motion components (u = east, v = north)
        let storm_u = self.storm_speed_mps * storm_to_rad.sin();
        let storm_v = self.storm_speed_mps * storm_to_rad.cos();

        for (az_idx, azimuth) in input.azimuth_angles_degrees().iter().enumerate() {
            let azimuth_rad = azimuth.to_radians();

            // Unit vector along the radial direction (from radar toward target)
            let radial_u = azimuth_rad.sin();
            let radial_v = azimuth_rad.cos();

            // Component of storm motion along this radial
            let storm_radial_component = storm_u * radial_u + storm_v * radial_v;

            let (Some(values), Some(statuses)) =
                (input.radial_values(az_idx), input.radial_statuses(az_idx))
            else {
                continue;
            };
            if let Some(output_statuses) = output.radial_statuses_mut(az_idx) {
                output_statuses.copy_from_slice(statuses);
            }
            let Some(output_values) = output.radial_values_mut(az_idx) else {
                continue;
            };

            // Subtract the storm motion component from valid gates
            for ((output_value, value), status) in
                output_values.iter_mut().zip(values).zip(statuses)
            {
                *output_value = if *status == GateStatus::Valid {
                    value - storm_radial_component
                } else {
                    *value
                };
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_velocity_field() -> SweepField {
        let azimuths = vec![0.0, 90.0, 180.0, 270.0];
        let gate_count = 5;

        let mut field = SweepField::builder("Velocity", "m/s")
            .product(Product::Velocity)
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(azimuths)
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(gate_count)
            .build()
            .expect("valid geometry");

        // Set uniform velocity of 10 m/s at all azimuths
        for az in 0..4 {
            for gate in 0..gate_count {
                field.set(az, gate, 10.0, GateStatus::Valid).unwrap();
            }
        }

        field
    }

    #[test]
    fn test_srv_zero_storm_motion() {
        let field = make_velocity_field();
        let srv = StormRelativeVelocity::new(0.0, 0.0).unwrap();
        let result = srv.process(&field).unwrap();

        // No storm motion — output should match input
        for az in 0..4 {
            for gate in 0..5 {
                let (val, _) = result.get(az, gate).unwrap();
                assert!((val - 10.0).abs() < 0.01);
            }
        }
    }

    #[test]
    fn test_srv_northward_storm() {
        let field = make_velocity_field();
        // Storm moving FROM south (180°), so moving TO north
        let srv = StormRelativeVelocity::new(180.0, 10.0).unwrap();
        let result = srv.process(&field).unwrap();

        // At azimuth 0° (north): radial is along storm direction
        // Storm component along north radial = +10 m/s
        // SR velocity = 10 - 10 = 0
        let (val, _) = result.get(0, 2).unwrap();
        assert!(val.abs() < 0.1, "Expected ~0 at north azimuth, got {}", val);

        // At azimuth 180° (south): radial is opposite storm direction
        // Storm component along south radial = -10 m/s
        // SR velocity = 10 - (-10) = 20
        let (val, _) = result.get(2, 2).unwrap();
        assert!(
            (val - 20.0).abs() < 0.1,
            "Expected ~20 at south azimuth, got {}",
            val
        );

        // At azimuth 90° (east): radial is perpendicular to storm direction
        // Storm component along east radial = 0
        // SR velocity = 10 - 0 = 10
        let (val, _) = result.get(1, 2).unwrap();
        assert!(
            (val - 10.0).abs() < 0.1,
            "Expected ~10 at east azimuth, got {}",
            val
        );
    }

    #[test]
    fn test_srv_preserves_nodata() {
        let mut field = make_velocity_field();
        field.set(1, 2, 0.0, GateStatus::NoData).unwrap();

        let srv = StormRelativeVelocity::new(180.0, 10.0).unwrap();
        let result = srv.process(&field).unwrap();

        let (_, status) = result.get(1, 2).unwrap();
        assert_eq!(status, GateStatus::NoData);
    }

    #[test]
    fn test_srv_preserves_filtered() {
        let mut field = make_velocity_field();
        field.set(2, 2, 10.0, GateStatus::Filtered).unwrap();

        let srv = StormRelativeVelocity::new(180.0, 10.0).unwrap();
        let result = srv.process(&field).unwrap();

        // A filtered gate keeps its original value rather than being adjusted.
        assert_eq!(result.get(2, 2), Some((10.0, GateStatus::Filtered)));
    }

    #[test]
    fn test_srv_parameter_validation() {
        let rejected =
            |direction: f32, speed: f32| match StormRelativeVelocity::new(direction, speed) {
                Err(Error::InvalidParameter { parameter, .. }) => parameter,
                other => panic!("expected InvalidParameter, got {other:?}"),
            };
        assert_eq!(rejected(0.0, -5.0), "storm_speed_mps");
        assert_eq!(rejected(0.0, f32::NAN), "storm_speed_mps");
        assert_eq!(rejected(0.0, f32::INFINITY), "storm_speed_mps");
        assert_eq!(rejected(f32::NAN, 5.0), "storm_direction_degrees");
        assert_eq!(rejected(360.0, 5.0), "storm_direction_degrees");
        assert_eq!(rejected(-1.0, 5.0), "storm_direction_degrees");

        let srv = StormRelativeVelocity::new(359.5, 0.0).unwrap();
        assert_eq!(srv.storm_direction_degrees(), 359.5);
        assert_eq!(srv.storm_speed_mps(), 0.0);
    }

    #[test]
    fn test_srv_output_label_and_product() -> Result<()> {
        let field = make_velocity_field();
        let result = StormRelativeVelocity::new(180.0, 10.0)?.process(&field)?;

        assert_eq!(result.label(), "Storm-Relative Velocity");
        assert_eq!(result.unit(), "m/s");
        assert_eq!(result.product(), None);
        assert_eq!(
            result.azimuth_angles_degrees(),
            field.azimuth_angles_degrees()
        );
        assert_eq!(result.gate_count(), field.gate_count());
        assert_eq!(result.first_gate_range_km(), field.first_gate_range_km());
        assert_eq!(result.gate_interval_km(), field.gate_interval_km());
        Ok(())
    }

    #[test]
    fn test_srv_requires_velocity() {
        let srv = StormRelativeVelocity::new(180.0, 10.0).unwrap();
        let reflectivity = SweepField::builder("Reflectivity", "dBZ")
            .product(Product::Reflectivity)
            .azimuth_angles_degrees(vec![0.0, 1.0])
            .gate_interval_km(0.25)
            .gate_count(4)
            .build()
            .unwrap();
        assert_eq!(
            srv.process(&reflectivity),
            Err(Error::UnexpectedProduct {
                expected: Product::Velocity,
                actual: Some(Product::Reflectivity),
            })
        );
    }
}
