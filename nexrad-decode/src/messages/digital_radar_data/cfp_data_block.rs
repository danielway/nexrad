use super::GenericDataBlock;
use std::ops::Deref;

/// CFP status codes for clutter filter power moments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// A decoded CFP gate value. Raw values 0–7 are status codes; values 8+ are numeric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaledCFPValue {
    /// A CFP status code (raw values 0–7).
    Status(CFPStatus),
    /// A decoded floating-point CFP value (raw values 8+).
    Value(f32),
}

/// A clutter filter power (CFP) data moment block.
///
/// This is a thin wrapper around [`GenericDataBlock`] that provides CFP-aware decoding.
/// All header and raw data accessors are available through [`Deref`]. The key difference
/// is that [`decoded_values`](CFPDataBlock::decoded_values) interprets raw values 0–7 as
/// CFP status codes rather than generic below-threshold/range-folded/numeric values.
#[derive(Clone, PartialEq, Debug)]
pub struct CFPDataBlock<'a> {
    inner: GenericDataBlock<'a>,
}

impl<'a> CFPDataBlock<'a> {
    /// Creates a new CFP data block wrapping a generic data block.
    pub(crate) fn new(inner: GenericDataBlock<'a>) -> Self {
        Self { inner }
    }

    /// Convert this data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> CFPDataBlock<'static> {
        CFPDataBlock {
            inner: self.inner.into_owned(),
        }
    }

    /// Decodes raw moment values using CFP-specific rules.
    ///
    /// Raw values 0–7 are decoded as CFP status codes. Values 8+ are decoded as
    /// floating-point values using the block's scale and offset.
    pub fn decoded_values(&self) -> Vec<ScaledCFPValue> {
        let scale = self.inner.header.scale();
        let offset = self.inner.header.offset();

        self.inner.decode_with(|raw_value| match raw_value {
            0 => ScaledCFPValue::Status(CFPStatus::FilterNotApplied),
            1 => ScaledCFPValue::Status(CFPStatus::PointClutterFilterApplied),
            2 => ScaledCFPValue::Status(CFPStatus::DualPolOnlyFilterApplied),
            3..=7 => ScaledCFPValue::Status(CFPStatus::Reserved(raw_value as u8)),
            _ => {
                if scale == 0.0 {
                    ScaledCFPValue::Value(raw_value as f32)
                } else {
                    ScaledCFPValue::Value((raw_value as f32 - offset) / scale)
                }
            }
        })
    }

    /// Get moment data from this CFP data block. Note that this will clone the underlying data.
    #[cfg(feature = "nexrad-model")]
    pub fn moment_data(&self) -> nexrad_model::data::CFPMomentData {
        nexrad_model::data::CFPMomentData::new(self.inner.moment_data_block())
    }

    /// Convert this CFP data block into common model moment data.
    #[cfg(feature = "nexrad-model")]
    pub fn into_moment_data(self) -> nexrad_model::data::CFPMomentData {
        nexrad_model::data::CFPMomentData::new(self.inner.into_moment_data_block())
    }
}

impl<'a> Deref for CFPDataBlock<'a> {
    type Target = GenericDataBlock<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_data::BinaryData;
    use crate::messages::digital_radar_data::raw;
    use crate::messages::digital_radar_data::GenericDataBlockHeader;
    use crate::messages::primitive_aliases::{Integer2, Integer4, Real4, ScaledInteger2};
    use std::borrow::Cow;

    #[test]
    fn test_decoded_values_cfp_status_codes() {
        let raw_header = raw::GenericDataBlockHeader {
            reserved: Integer4::new(0),
            number_of_data_moment_gates: Integer2::new(6),
            data_moment_range: ScaledInteger2::new(0),
            data_moment_range_sample_interval: ScaledInteger2::new(1),
            tover: ScaledInteger2::new(0),
            snr_threshold: ScaledInteger2::new(0),
            control_flags: 0,
            data_word_size: 8,
            scale: Real4::new(1.0),
            offset: Real4::new(0.0),
        };

        let header = GenericDataBlockHeader::new(&raw_header);
        let encoded = vec![0, 1, 2, 3, 7, 8];
        let block = CFPDataBlock::new(GenericDataBlock {
            header,
            encoded_data: BinaryData::new(Cow::Owned(encoded)),
        });

        let decoded = block.decoded_values();
        assert_eq!(
            decoded[0],
            ScaledCFPValue::Status(CFPStatus::FilterNotApplied)
        );
        assert_eq!(
            decoded[1],
            ScaledCFPValue::Status(CFPStatus::PointClutterFilterApplied)
        );
        assert_eq!(
            decoded[2],
            ScaledCFPValue::Status(CFPStatus::DualPolOnlyFilterApplied)
        );
        assert_eq!(
            decoded[3],
            ScaledCFPValue::Status(CFPStatus::Reserved(3))
        );
        assert_eq!(
            decoded[4],
            ScaledCFPValue::Status(CFPStatus::Reserved(7))
        );
        assert_eq!(decoded[5], ScaledCFPValue::Value(8.0));
    }
}
