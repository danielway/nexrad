use crate::result::{Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, SweepField};

/// Storm-relative velocity — subtracts a storm motion vector from radial velocity.
///
/// This makes it easier to identify rotation signatures (mesocyclones) in storm
/// systems by removing the bulk storm motion. The input field should contain
/// radial velocity data.
///
/// The storm motion is specified as a direction (degrees from north) and speed (m/s).
/// The component of the storm motion along each gate's radial direction is subtracted
/// from the measured velocity.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::derived::StormRelativeVelocity;
///
/// // Storm moving from 240° at 15 m/s
/// let srv = StormRelativeVelocity::new(240.0, 15.0)?;
/// let sr_velocity = srv.process(&velocity_field)?;
/// ```
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
    /// - `storm_direction_degrees` — Direction the storm is moving FROM, in degrees
    ///   clockwise from north (0-360).
    /// - `storm_speed_mps` — Storm movement speed in m/s.
    ///
    /// # Errors
    ///
    /// Returns an error if speed is negative.
    pub fn new(storm_direction_degrees: f32, storm_speed_mps: f32) -> Result<Self> {
        if storm_speed_mps < 0.0 {
            return Err(Error::InvalidParameter(
                "storm speed must be non-negative".to_string(),
            ));
        }
        Ok(Self {
            storm_direction_degrees,
            storm_speed_mps,
        })
    }
}

impl SweepProcessor for StormRelativeVelocity {
    fn name(&self) -> &str {
        "StormRelativeVelocity"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        let mut output = input.clone();

        // Convert storm direction to radians (meteorological convention: FROM direction)
        // The storm motion vector points in the direction the storm is moving TO.
        let storm_to_rad = (self.storm_direction_degrees + 180.0).to_radians();

        // Storm motion components (u = east, v = north)
        let storm_u = self.storm_speed_mps * storm_to_rad.sin();
        let storm_v = self.storm_speed_mps * storm_to_rad.cos();

        for az_idx in 0..input.azimuth_count() {
            let azimuth_rad = input.azimuths()[az_idx].to_radians();

            // Unit vector along the radial direction (from radar toward target)
            let radial_u = azimuth_rad.sin();
            let radial_v = azimuth_rad.cos();

            // Component of storm motion along this radial
            let storm_radial_component = storm_u * radial_u + storm_v * radial_v;

            for gate_idx in 0..input.gate_count() {
                let (val, status) = input.get(az_idx, gate_idx);
                if status != GateStatus::Valid {
                    continue;
                }

                // Subtract storm motion component
                let sr_velocity = val - storm_radial_component;
                output.set(az_idx, gate_idx, sr_velocity, GateStatus::Valid);
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

        let mut field =
            SweepField::new_empty("Velocity", "m/s", 0.5, azimuths, 1.0, 2.0, 0.25, gate_count);

        // Set uniform velocity of 10 m/s at all azimuths
        for az in 0..4 {
            for gate in 0..gate_count {
                field.set(az, gate, 10.0, GateStatus::Valid);
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
                let (val, _) = result.get(az, gate);
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
        let (val, _) = result.get(0, 2);
        assert!(val.abs() < 0.1, "Expected ~0 at north azimuth, got {}", val);

        // At azimuth 180° (south): radial is opposite storm direction
        // Storm component along south radial = -10 m/s
        // SR velocity = 10 - (-10) = 20
        let (val, _) = result.get(2, 2);
        assert!(
            (val - 20.0).abs() < 0.1,
            "Expected ~20 at south azimuth, got {}",
            val
        );

        // At azimuth 90° (east): radial is perpendicular to storm direction
        // Storm component along east radial = 0
        // SR velocity = 10 - 0 = 10
        let (val, _) = result.get(1, 2);
        assert!(
            (val - 10.0).abs() < 0.1,
            "Expected ~10 at east azimuth, got {}",
            val
        );
    }

    #[test]
    fn test_srv_preserves_nodata() {
        let mut field = make_velocity_field();
        field.set(1, 2, 0.0, GateStatus::NoData);

        let srv = StormRelativeVelocity::new(180.0, 10.0).unwrap();
        let result = srv.process(&field).unwrap();

        let (_, status) = result.get(1, 2);
        assert_eq!(status, GateStatus::NoData);
    }

    #[test]
    fn test_srv_negative_speed_error() {
        assert!(StormRelativeVelocity::new(0.0, -5.0).is_err());
    }
}
