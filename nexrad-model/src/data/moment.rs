use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "uom")]
use uom::si::{f64::Length, length::kilometer};

/// Moment data from a radial for a particular product where each value corresponds to a gate.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MomentData {
    gate_count: u16,
    first_gate_range: u16,
    gate_interval: u16,
    /// Bits per gate (8 or 16). Dual-pol moments often use 16-bit words.
    data_word_size: u8,
    scale: f32,
    offset: f32,
    values: Vec<u8>,
}

impl MomentData {
    /// Create new moment data from fixed-point encoding.
    pub fn from_fixed_point(
        gate_count: u16,
        first_gate_range: u16,
        gate_interval: u16,
        data_word_size: u8,
        scale: f32,
        offset: f32,
        values: Vec<u8>,
    ) -> Self {
        Self {
            gate_count,
            first_gate_range,
            gate_interval,
            data_word_size,
            scale,
            offset,
            values,
        }
    }

    /// The number of gates in this data moment.
    pub fn gate_count(&self) -> u16 {
        self.gate_count
    }

    /// The range to the center of the first gate in kilometers.
    pub fn first_gate_range_km(&self) -> f64 {
        self.first_gate_range as f64 * 0.001
    }

    /// The range to the center of the first gate.
    #[cfg(feature = "uom")]
    pub fn first_gate_range(&self) -> Length {
        Length::new::<kilometer>(self.first_gate_range as f64 * 0.001)
    }

    /// The range between the centers of consecutive gates in kilometers.
    pub fn gate_interval_km(&self) -> f64 {
        self.gate_interval as f64 * 0.001
    }

    /// Number of bits per gate (8 or 16).
    pub fn data_word_size(&self) -> u8 {
        self.data_word_size
    }

    /// The range between the centers of consecutive gates.
    #[cfg(feature = "uom")]
    pub fn gate_interval(&self) -> Length {
        Length::new::<kilometer>(self.gate_interval as f64 * 0.001)
    }

    /// Values from this data moment corresponding to gates in the radial.
    pub fn values(&self) -> Vec<MomentValue> {
        let scale = self.scale;
        let offset = self.offset;

        let decode = |raw_value: u16| {
            if scale == 0.0 {
                return MomentValue::Value(raw_value as f32);
            }

            match raw_value {
                0 => MomentValue::BelowThreshold,
                1 => MomentValue::RangeFolded,
                _ => MomentValue::Value((raw_value as f32 - offset) / scale),
            }
        };

        if self.data_word_size == 16 {
            // 16-bit moments store big-endian u16 values per gate.
            self.values
                .chunks_exact(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                .map(decode)
                .collect()
        } else {
            // Default to 8-bit decoding.
            self.values.iter().copied().map(|v| decode(v as u16)).collect()
        }
    }
}

impl Debug for MomentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MomentData")
            .field("data_word_size", &self.data_word_size)
            .field("values", &self.values())
            .finish()
    }
}

/// The data moment value for a product in a radial's gate. The value may be a floating-point number
/// or a special case such as "below threshold" or "range folded".
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MomentValue {
    /// The data moment value for a gate.
    Value(f32),
    /// The value for this gate was below the signal threshold.
    BelowThreshold,
    /// The value for this gate exceeded the maximum unambiguous range.
    RangeFolded,
}
