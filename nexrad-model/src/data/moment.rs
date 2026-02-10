use std::fmt::Debug;
use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "uom")]
use uom::si::{f64::Length, length::kilometer};

/// CFP status codes for clutter filter power moments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CFPStatus {
    /// Clutter filter not applied.
    FilterNotApplied,
    /// Point clutter filter applied.
    PointClutterFilterApplied,
    /// Dual-pol-only filter applied.
    DualPolOnlyFilterApplied,
    /// Reserved CFP status code.
    Reserved(u8),
}

/// Encoded moment data from a radial containing gate metadata and raw values.
///
/// This type provides gate metadata accessors (count, range, interval) shared by both
/// generic moments and CFP moments. It does not decode values — use [`MomentData`] or
/// [`CFPMomentData`] for decoded gate values.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MomentDataBlock {
    gate_count: u16,
    first_gate_range: u16,
    gate_interval: u16,
    /// Bits per gate (8 or 16). Dual-pol moments often use 16-bit words.
    data_word_size: u8,
    scale: f32,
    offset: f32,
    values: Vec<u8>,
}

impl MomentDataBlock {
    /// Create new moment data block from fixed-point encoding.
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

    pub(crate) fn decode_with<T>(&self, decode: impl Fn(u16) -> T) -> Vec<T> {
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

impl Debug for MomentDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MomentDataBlock")
            .field("gate_count", &self.gate_count)
            .field("first_gate_range", &self.first_gate_range)
            .field("gate_interval", &self.gate_interval)
            .field("data_word_size", &self.data_word_size)
            .finish()
    }
}

/// Moment data from a radial for a particular product where each value corresponds to a gate.
///
/// Gate metadata (count, range, interval) is available through [`Deref<Target = MomentDataBlock>`].
/// Use [`values`](MomentData::values) to decode gates with standard moment semantics
/// (below threshold, range folded, or numeric value).
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct MomentData {
    inner: MomentDataBlock,
}

impl MomentData {
    /// Create new moment data wrapping a data block.
    pub fn new(inner: MomentDataBlock) -> Self {
        Self { inner }
    }

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
            inner: MomentDataBlock::from_fixed_point(
                gate_count,
                first_gate_range,
                gate_interval,
                data_word_size,
                scale,
                offset,
                values,
            ),
        }
    }

    /// Values from this data moment corresponding to gates in the radial.
    pub fn values(&self) -> Vec<MomentValue> {
        let scale = self.inner.scale;
        let offset = self.inner.offset;

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

        self.inner.decode_with(decode)
    }
}

impl Deref for MomentData {
    type Target = MomentDataBlock;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for MomentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MomentData")
            .field("data_word_size", &self.inner.data_word_size)
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

/// A decoded CFP gate value. Raw values 0–7 are status codes; values 8+ are numeric.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CFPMomentValue {
    /// A CFP status code (raw values 0–7).
    Status(CFPStatus),
    /// A decoded floating-point CFP value (raw values 8+).
    Value(f32),
}

/// Clutter filter power (CFP) moment data wrapping a [`MomentDataBlock`].
///
/// Gate metadata (count, range, interval) is available through [`Deref<Target = MomentDataBlock>`].
/// Use [`values`](CFPMomentData::values) to decode gates with CFP-specific semantics.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct CFPMomentData {
    inner: MomentDataBlock,
}

impl CFPMomentData {
    /// Create a new CFP moment data wrapper.
    pub fn new(inner: MomentDataBlock) -> Self {
        Self { inner }
    }

    /// Create new CFP moment data from fixed-point encoding.
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
            inner: MomentDataBlock::from_fixed_point(
                gate_count,
                first_gate_range,
                gate_interval,
                data_word_size,
                scale,
                offset,
                values,
            ),
        }
    }

    /// Decode gate values with CFP-specific rules.
    ///
    /// Raw values 0–7 are decoded as CFP status codes. Values 8+ are decoded as
    /// floating-point values using the moment's scale and offset.
    pub fn values(&self) -> Vec<CFPMomentValue> {
        let scale = self.inner.scale;
        let offset = self.inner.offset;

        self.inner.decode_with(|raw_value| match raw_value {
            0 => CFPMomentValue::Status(CFPStatus::FilterNotApplied),
            1 => CFPMomentValue::Status(CFPStatus::PointClutterFilterApplied),
            2 => CFPMomentValue::Status(CFPStatus::DualPolOnlyFilterApplied),
            3..=7 => CFPMomentValue::Status(CFPStatus::Reserved(raw_value as u8)),
            _ => {
                if scale == 0.0 {
                    CFPMomentValue::Value(raw_value as f32)
                } else {
                    CFPMomentValue::Value((raw_value as f32 - offset) / scale)
                }
            }
        })
    }
}

impl Deref for CFPMomentData {
    type Target = MomentDataBlock;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for CFPMomentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CFPMomentData")
            .field("data_word_size", &self.inner.data_word_size)
            .field("values", &self.values())
            .finish()
    }
}
