use crate::binary_data::BinaryData;
use crate::data::Product;
use crate::result::{Error, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// CFP status codes for clutter filter power moments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
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
///
/// Each instance knows which [`Product`] it holds, which determines how raw gate values are
/// decoded by [`iter`](Self::iter) and [`values`](Self::values):
///
/// - For [`Product::ClutterFilterPower`], raw values 0–7 are CFP status codes and decode to
///   [`MomentValue::CfpStatus`]; larger values decode to [`MomentValue::Value`].
/// - For every other product, raw value 0 decodes to [`MomentValue::BelowThreshold`], raw value
///   1 decodes to [`MomentValue::RangeFolded`], and larger values decode to
///   [`MomentValue::Value`].
///
/// A scale of `0.0` means numeric raw values are used directly; otherwise they are decoded as
/// `(raw - offset) / scale`. The raw encoding parameters and bytes are also exposed for
/// consumers that decode gate values themselves.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct MomentData {
    product: Product,
    gate_count: u16,
    first_gate_range_meters: u16,
    gate_interval_meters: u16,
    /// Bits per gate (8 or 16). Dual-pol moments often use 16-bit words.
    data_word_size: u8,
    scale: f32,
    offset: f32,
    raw_values: BinaryData<Vec<u8>>,
}

impl MomentData {
    /// Start building moment data for `product` from its raw encoded gate bytes.
    ///
    /// The encoding parameters take the defaults documented on [`MomentDataBuilder`]; set the
    /// gate geometry and the scale and offset of real data explicitly.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{MomentData, MomentValue, Product};
    ///
    /// let moment = MomentData::builder(Product::Reflectivity, vec![0, 1, 100])
    ///     .first_gate_range_meters(2125)
    ///     .gate_interval_meters(250)
    ///     .scale(2.0)
    ///     .offset(66.0)
    ///     .build()?;
    /// assert_eq!(moment.gate_count(), 3);
    /// assert_eq!(
    ///     moment.values(),
    ///     vec![MomentValue::BelowThreshold, MomentValue::RangeFolded, MomentValue::Value(17.0)],
    /// );
    ///
    /// // A declared gate count must match the bytes given.
    /// assert!(MomentData::builder(Product::Reflectivity, vec![0, 1, 100])
    ///     .gate_count(4)
    ///     .build()
    ///     .is_err());
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn builder(product: Product, raw_values: Vec<u8>) -> MomentDataBuilder {
        MomentDataBuilder {
            product,
            raw_values,
            gate_count: None,
            first_gate_range_meters: 0,
            gate_interval_meters: 0,
            data_word_size: 8,
            scale: 0.0,
            offset: 0.0,
        }
    }

    /// The product this moment data holds.
    pub fn product(&self) -> Product {
        self.product
    }

    /// The number of gates in this data moment.
    pub fn gate_count(&self) -> usize {
        usize::from(self.gate_count)
    }

    /// The range to the center of the first gate in meters.
    pub fn first_gate_range_meters(&self) -> u16 {
        self.first_gate_range_meters
    }

    /// The range between the centers of consecutive gates in meters.
    pub fn gate_interval_meters(&self) -> u16 {
        self.gate_interval_meters
    }

    /// The range to the center of the first gate in kilometers.
    pub fn first_gate_range_km(&self) -> f64 {
        self.first_gate_range_meters as f64 * 0.001
    }

    /// The range between the centers of consecutive gates in kilometers.
    pub fn gate_interval_km(&self) -> f64 {
        self.gate_interval_meters as f64 * 0.001
    }

    /// Number of bits per gate (8 or 16).
    pub fn data_word_size(&self) -> u8 {
        self.data_word_size
    }

    /// The scale factor used to decode raw gate values into floating-point values.
    /// A value of `0.0` means raw values are used directly without scaling.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// The offset used to decode raw gate values into floating-point values.
    /// The decoded value is `(raw - offset) / scale`.
    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// The raw encoded gate values as bytes. For 8-bit moments, each byte is one gate.
    /// For 16-bit moments, each pair of bytes is a big-endian `u16` gate value.
    pub fn raw_values(&self) -> &[u8] {
        &self.raw_values
    }

    /// Iterator over raw gate values as `u16`, handling both 8-bit and 16-bit word sizes.
    pub fn raw_gate_values(&self) -> impl Iterator<Item = u16> + '_ {
        let is_16bit = self.data_word_size == 16;
        let step = if is_16bit { 2 } else { 1 };
        self.raw_values.chunks_exact(step).map(move |chunk| {
            if is_16bit {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                chunk[0] as u16
            }
        })
    }

    /// Iterator over decoded gate values without allocating.
    ///
    /// See the [type-level documentation](Self) for how raw values are decoded per product.
    pub fn iter(&self) -> impl Iterator<Item = MomentValue> + '_ {
        let scale = self.scale;
        let offset = self.offset;
        let is_cfp = self.product == Product::ClutterFilterPower;

        self.raw_gate_values().map(move |raw_value| {
            if is_cfp {
                match raw_value {
                    0 => return MomentValue::CfpStatus(CfpStatus::FilterNotApplied),
                    1 => return MomentValue::CfpStatus(CfpStatus::PointClutterFilterApplied),
                    2 => return MomentValue::CfpStatus(CfpStatus::DualPolOnlyFilterApplied),
                    3..=7 => return MomentValue::CfpStatus(CfpStatus::Reserved(raw_value as u8)),
                    _ => {}
                }
            }

            if !is_cfp {
                match raw_value {
                    0 => return MomentValue::BelowThreshold,
                    1 => return MomentValue::RangeFolded,
                    _ => {}
                }
            }

            // scale == 0.0 is an exact comparison; the value comes from a binary format
            // where IEEE 754 zero is stored literally.
            if scale == 0.0 {
                return MomentValue::Value(raw_value as f32);
            }

            MomentValue::Value((raw_value as f32 - offset) / scale)
        })
    }

    /// Decoded gate values collected into a `Vec`. Prefer [`iter`](Self::iter) when
    /// processing values sequentially to avoid allocation.
    pub fn values(&self) -> Vec<MomentValue> {
        self.iter().collect()
    }
}

/// Builder for [`MomentData`], created with [`MomentData::builder`].
///
/// The product and the raw gate bytes are required. All other properties are optional and
/// default to:
///
/// - `gate_count`: the number of gates `raw_values` holds at the word size
/// - `first_gate_range_meters`: `0`
/// - `gate_interval_meters`: `0`
/// - `data_word_size`: `8`
/// - `scale`: `0.0` (raw values are used directly)
/// - `offset`: `0.0`
#[derive(Debug, Clone)]
#[must_use]
pub struct MomentDataBuilder {
    product: Product,
    raw_values: Vec<u8>,
    gate_count: Option<usize>,
    first_gate_range_meters: u16,
    gate_interval_meters: u16,
    data_word_size: u8,
    scale: f32,
    offset: f32,
}

impl MomentDataBuilder {
    /// Declare the number of gates, which [`build`](Self::build) checks against the length of
    /// the raw values. Defaults to the number of gates the raw values hold.
    pub fn gate_count(mut self, gate_count: usize) -> Self {
        self.gate_count = Some(gate_count);
        self
    }

    /// Set the range to the center of the first gate in meters. Defaults to `0`.
    pub fn first_gate_range_meters(mut self, first_gate_range_meters: u16) -> Self {
        self.first_gate_range_meters = first_gate_range_meters;
        self
    }

    /// Set the range between the centers of consecutive gates in meters. Defaults to `0`.
    pub fn gate_interval_meters(mut self, gate_interval_meters: u16) -> Self {
        self.gate_interval_meters = gate_interval_meters;
        self
    }

    /// Set the number of bits per gate, 8 or 16. 16-bit gate values are stored big-endian in
    /// the raw values. Defaults to `8`.
    pub fn data_word_size(mut self, data_word_size: u8) -> Self {
        self.data_word_size = data_word_size;
        self
    }

    /// Set the scale used to decode raw gate values as `(raw - offset) / scale`. A scale of
    /// `0.0` means raw values are used directly. Defaults to `0.0`.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set the offset used to decode raw gate values as `(raw - offset) / scale`. Defaults to
    /// `0.0`.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Build the [`MomentData`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidMomentData`] if `data_word_size` is not 8 or 16, the gate count
    /// does not fit in a `u16`, or `scale` or `offset` is not finite, and
    /// [`Error::MomentDataLength`] if a declared gate count does not match the raw values, or
    /// the raw values do not hold a whole number of 16-bit gates.
    pub fn build(self) -> Result<MomentData> {
        let bytes_per_gate = match self.data_word_size {
            8 => 1,
            16 => 2,
            _ => {
                return Err(Error::InvalidMomentData {
                    parameter: "data_word_size",
                    reason: "must be 8 or 16",
                })
            }
        };
        let gate_count = self
            .gate_count
            .unwrap_or(self.raw_values.len() / bytes_per_gate);
        let compact_gate_count =
            u16::try_from(gate_count).map_err(|_| Error::InvalidMomentData {
                parameter: "gate_count",
                reason: "must fit in a u16",
            })?;
        let expected_len = gate_count * bytes_per_gate;
        if self.raw_values.len() != expected_len {
            return Err(Error::MomentDataLength {
                expected: expected_len,
                actual: self.raw_values.len(),
            });
        }
        for (parameter, value) in [("scale", self.scale), ("offset", self.offset)] {
            if !value.is_finite() {
                return Err(Error::InvalidMomentData {
                    parameter,
                    reason: "must be finite",
                });
            }
        }

        Ok(MomentData {
            product: self.product,
            gate_count: compact_gate_count,
            first_gate_range_meters: self.first_gate_range_meters,
            gate_interval_meters: self.gate_interval_meters,
            data_word_size: self.data_word_size,
            scale: self.scale,
            offset: self.offset,
            raw_values: BinaryData::new(self.raw_values),
        })
    }
}

/// Deserializes through [`MomentDataBuilder::build`], so invalid encoding parameters or a
/// byte length that does not match the gate count and word size are rejected.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MomentData {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let repr = MomentDataRepr::deserialize(deserializer)?;
        MomentData::builder(repr.product, repr.raw_values.0)
            .gate_count(usize::from(repr.gate_count))
            .first_gate_range_meters(repr.first_gate_range_meters)
            .gate_interval_meters(repr.gate_interval_meters)
            .data_word_size(repr.data_word_size)
            .scale(repr.scale)
            .offset(repr.offset)
            .build()
            .map_err(serde::de::Error::custom)
    }
}

/// The serialized form of [`MomentData`], validated before it becomes one.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(rename = "MomentData")]
struct MomentDataRepr {
    product: Product,
    gate_count: u16,
    first_gate_range_meters: u16,
    gate_interval_meters: u16,
    data_word_size: u8,
    scale: f32,
    offset: f32,
    raw_values: BinaryData<Vec<u8>>,
}

/// The data moment value for a product in a radial's gate. The value may be a floating-point
/// number or a special case such as "below threshold", "range folded", or a clutter filter
/// power status code.
///
/// This enum is exhaustive: its variants are the cases of the ICD's gate encoding, so a new
/// variant would only follow a change to that encoding and is reserved for a major release.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MomentValue {
    /// The data moment value for a gate.
    Value(f32),
    /// The value for this gate was below the signal threshold.
    BelowThreshold,
    /// The value for this gate exceeded the maximum unambiguous range.
    RangeFolded,
    /// A clutter filter power status code. Only produced for
    /// [`Product::ClutterFilterPower`] moments, for raw values 0–7.
    CfpStatus(CfpStatus),
}

/// The validity status of a gate value in a field.
///
/// Used alongside `f32` values in field types ([`SweepField`](crate::data::SweepField),
/// [`CartesianField`](crate::data::CartesianField), [`VerticalField`](crate::data::VerticalField))
/// to indicate whether each gate contains a valid measurement or a special condition.
///
/// This enum is `#[non_exhaustive]`: [`Filtered`](Self::Filtered) and [`NoData`](Self::NoData)
/// are set by processing steps rather than by the data encoding, and a later processing step may
/// add a status in a minor release. Matches outside this crate need a wildcard (`_`) arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum GateStatus {
    /// The gate contains a valid floating-point value.
    Valid,
    /// The signal was below the detection threshold.
    BelowThreshold,
    /// The gate exceeded the maximum unambiguous range (a second-trip echo), so its value is
    /// unavailable.
    RangeFolded,
    /// No data is available for this gate.
    NoData,
    /// The gate had a valid value that a processing step removed (for example a threshold or
    /// correlation-coefficient filter); the value is kept so consumers can still inspect it.
    Filtered,
}

impl From<MomentValue> for GateStatus {
    /// Maps a decoded moment value to its gate status. Clutter filter power status codes carry
    /// no measurement and map to [`GateStatus::NoData`].
    fn from(value: MomentValue) -> Self {
        match value {
            MomentValue::Value(_) => GateStatus::Valid,
            MomentValue::BelowThreshold => GateStatus::BelowThreshold,
            MomentValue::RangeFolded => GateStatus::RangeFolded,
            MomentValue::CfpStatus(_) => GateStatus::NoData,
        }
    }
}
