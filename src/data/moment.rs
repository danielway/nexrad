use std::fmt::Debug;

/// Moment data from a radial for a particular product where each value corresponds to a gate.
pub struct MomentData {
    scale: f32,
    offset: f32,
    values: Vec<u8>,
}

impl MomentData {
    /// Create new moment data from fixed-point encoding.
    pub fn from_fixed_point(scale: f32, offset: f32, values: Vec<u8>) -> Self {
        Self {
            scale,
            offset,
            values,
        }
    }

    /// Values from this data moment corresponding to gates in the radial.
    pub fn values(&self) -> Vec<MomentValue> {
        let copied_values = self.values.iter().copied();

        if self.scale == 0.0 {
            return copied_values
                .map(|raw_value| MomentValue::Value(raw_value as f32))
                .collect();
        }

        copied_values
            .map(|raw_value| match raw_value {
                0 => MomentValue::BelowThreshold,
                1 => MomentValue::RangeFolded,
                _ => MomentValue::Value((raw_value as f32 - self.offset) / self.scale),
            })
            .collect()
    }
}

impl Debug for MomentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MomentData")
            .field("values", &self.values())
            .finish()
    }
}

/// The data moment value for a product in a radial's gate. The value may be a floating-point number
/// or a special case such as "below threshold" or "range folded".
#[derive(Debug)]
pub enum MomentValue {
    /// The data moment value for a gate.
    Value(f32),
    /// The value for this gate was below the signal threshold.
    BelowThreshold,
    /// The value for this gate exceeded the maximum unambiguous range.
    RangeFolded,
}
