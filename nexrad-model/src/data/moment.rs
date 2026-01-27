use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "uom")]
use uom::si::{f64::Length, length::kilometer};

/// Describes the data moment represented by a [`MomentData`] instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MomentDataKind {
    /// Base reflectivity (dBZ).
    Reflectivity,
    /// Radial velocity (m/s).
    Velocity,
    /// Spectrum width (m/s).
    SpectrumWidth,
    /// Differential reflectivity (dB).
    DifferentialReflectivity,
    /// Differential phase (degrees).
    DifferentialPhase,
    /// Correlation coefficient (unitless).
    CorrelationCoefficient,
    /// Clutter filter power (CFP).
    ClutterFilterPower,
    /// Unknown or unspecified moment type.
    Unknown,
}

impl Default for MomentDataKind {
    fn default() -> Self {
        Self::Unknown
    }
}

/// CFP status codes for clutter filter power moments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CfpStatus {
    /// Clutter filter not applied.
    FilterNotApplied,
    /// Point clutter filter applied.
    PointClutterFilterApplied,
    /// Dual-pol-only filter applied.
    DualPolOnlyFilterApplied,
    /// Reserved CFP status code.
    Reserved(u8),
}

/// Moment data from a radial for a particular product where each value corresponds to a gate.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MomentData {
    /// The product type this moment represents.
    #[cfg_attr(feature = "serde", serde(default))]
    kind: MomentDataKind,
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
        kind: MomentDataKind,
        gate_count: u16,
        first_gate_range: u16,
        gate_interval: u16,
        data_word_size: u8,
        scale: f32,
        offset: f32,
        values: Vec<u8>,
    ) -> Self {
        Self {
            kind,
            gate_count,
            first_gate_range,
            gate_interval,
            data_word_size,
            scale,
            offset,
            values,
        }
    }

    /// The kind of data moment stored in this structure.
    pub fn kind(&self) -> MomentDataKind {
        self.kind
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

        let decode_generic = |raw_value: u16| {
            if scale == 0.0 {
                return MomentValue::Value(raw_value as f32);
            }

            match raw_value {
                0 => MomentValue::BelowThreshold,
                1 => MomentValue::RangeFolded,
                _ => MomentValue::Value((raw_value as f32 - offset) / scale),
            }
        };

        let decode_cfp = |raw_value: u16| {
            match raw_value {
                0 => MomentValue::CfpStatus(CfpStatus::FilterNotApplied),
                1 => MomentValue::CfpStatus(CfpStatus::PointClutterFilterApplied),
                2 => MomentValue::CfpStatus(CfpStatus::DualPolOnlyFilterApplied),
                3..=7 => MomentValue::CfpStatus(CfpStatus::Reserved(raw_value as u8)),
                _ => {
                    if scale == 0.0 {
                        MomentValue::Value(raw_value as f32)
                    } else {
                        MomentValue::Value((raw_value as f32 - offset) / scale)
                    }
                }
            }
        };

        let decode = |raw_value: u16| match self.kind {
            MomentDataKind::ClutterFilterPower => decode_cfp(raw_value),
            _ => decode_generic(raw_value),
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
            self.values
                .iter()
                .copied()
                .map(|v| decode(v as u16))
                .collect()
        }
    }
}

impl Debug for MomentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MomentData")
            .field("kind", &self.kind())
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
    /// CFP-specific status codes for clutter filter power moments.
    CfpStatus(CfpStatus),
}
