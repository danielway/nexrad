use super::raw;
use super::{CfpStatus, DataMomentGenericPointerType, GenericDataBlockHeader, ScaledMomentValue};
use crate::binary_data::BinaryData;
use crate::result::Result;
use crate::slice_reader::SliceReader;
use std::borrow::Cow;
use std::fmt::Debug;

/// A generic data moment block.
#[derive(Clone, PartialEq, Debug)]
pub struct GenericDataBlock<'a> {
    /// The generic data block's header information.
    header: GenericDataBlockHeader<'a>,

    /// The generic data block's encoded moment data.
    encoded_data: BinaryData<Cow<'a, [u8]>>,
}

impl<'a> GenericDataBlock<'a> {
    /// Creates a new generic data moment block from the decoded header.
    pub(crate) fn parse(reader: &mut SliceReader<'a>) -> Result<Self> {
        let raw_header = reader.take_ref::<raw::GenericDataBlockHeader>()?;

        let word_size_bytes = raw_header.data_word_size as usize / 8;
        let encoded_data_size =
            raw_header.number_of_data_moment_gates.get() as usize * word_size_bytes;

        let encoded_data = reader.take_bytes(encoded_data_size)?;

        Ok(Self {
            header: GenericDataBlockHeader::new(raw_header),
            encoded_data: BinaryData::new(Cow::Borrowed(encoded_data)),
        })
    }

    /// Convert this data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> GenericDataBlock<'static> {
        GenericDataBlock {
            header: self.header.into_owned(),
            encoded_data: BinaryData::new(Cow::Owned(self.encoded_data.0.into_owned())),
        }
    }

    /// The generic data block's header information.
    pub fn header(&self) -> &GenericDataBlockHeader<'a> {
        &self.header
    }

    /// The generic data block's encoded moment data.
    pub fn encoded_data(&self) -> &BinaryData<Cow<'a, [u8]>> {
        &self.encoded_data
    }

    /// Raw gate values for this moment/radial ordered in ascending distance from the radar. These
    /// values are stored in a fixed-point representation using the `DataMomentHeader.offset` and
    /// `DataMomentHeader.scale` fields. `decoded_data` provides decoded floating-point values.
    pub fn encoded_values(&self) -> &[u8] {
        &self.encoded_data
    }

    /// Decodes raw moment values from `encoded_data` from their fixed-point representation into
    /// their floating point representation. Additionally, identifies special values such as "below
    /// threshold" and "range folded".
    pub fn decoded_values(&self) -> Vec<ScaledMomentValue> {
        self.decode_generic()
    }

    /// Decodes raw moment values using the provided data moment type.
    ///
    /// CFP has special status codes (0..=7) and should not be decoded using the generic rules.
    pub fn decoded_values_with_type(
        &self,
        moment_type: DataMomentGenericPointerType,
    ) -> Vec<ScaledMomentValue> {
        match moment_type {
            DataMomentGenericPointerType::ClutterFilterPower => self.decode_cfp(),
            _ => self.decode_generic(),
        }
    }

    fn decode_generic(&self) -> Vec<ScaledMomentValue> {
        let scale = self.header.scale();
        let offset = self.header.offset();

        let decode = |raw_value: u16| {
            if scale == 0.0 {
                return ScaledMomentValue::Value(raw_value as f32);
            }

            match raw_value {
                0 => ScaledMomentValue::BelowThreshold,
                1 => ScaledMomentValue::RangeFolded,
                _ => ScaledMomentValue::Value((raw_value as f32 - offset) / scale),
            }
        };

        self.decode_with(decode)
    }

    fn decode_cfp(&self) -> Vec<ScaledMomentValue> {
        let scale = self.header.scale();
        let offset = self.header.offset();

        let decode = |raw_value: u16| match raw_value {
            0 => ScaledMomentValue::CfpStatus(CfpStatus::FilterNotApplied),
            1 => ScaledMomentValue::CfpStatus(CfpStatus::PointClutterFilterApplied),
            2 => ScaledMomentValue::CfpStatus(CfpStatus::DualPolOnlyFilterApplied),
            3..=7 => ScaledMomentValue::CfpStatus(CfpStatus::Reserved(raw_value as u8)),
            _ => {
                if scale == 0.0 {
                    ScaledMomentValue::Value(raw_value as f32)
                } else {
                    ScaledMomentValue::Value((raw_value as f32 - offset) / scale)
                }
            }
        };

        self.decode_with(decode)
    }

    fn decode_with(&self, decode: impl Fn(u16) -> ScaledMomentValue) -> Vec<ScaledMomentValue> {
        if self.header.data_word_size() == 16 {
            // 16-bit moments store big-endian u16 values per gate.
            self.encoded_data
                .chunks_exact(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                .map(decode)
                .collect()
        } else {
            // Default to 8-bit decoding.
            self.encoded_data
                .iter()
                .copied()
                .map(|v| decode(v as u16))
                .collect()
        }
    }

    /// Get moment data from this generic data block. Note that this will clone the underlying data.
    #[cfg(feature = "nexrad-model")]
    pub fn moment_data_with_kind(
        &self,
        kind: nexrad_model::data::MomentDataKind,
    ) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            kind,
            self.header.number_of_data_moment_gates(),
            self.header.data_moment_range_raw(),
            self.header.data_moment_range_sample_interval_raw(),
            self.header.data_word_size(),
            self.header.scale(),
            self.header.offset(),
            self.encoded_data.to_vec(),
        )
    }

    /// Deprecated alias that uses an unknown moment kind.
    #[cfg(feature = "nexrad-model")]
    #[deprecated(note = "use moment_data_with_kind for correct CFP decoding")]
    pub fn moment_data(&self) -> nexrad_model::data::MomentData {
        self.moment_data_with_kind(nexrad_model::data::MomentDataKind::Unknown)
    }

    /// Convert this generic data block into common model moment data.
    #[cfg(feature = "nexrad-model")]
    pub fn into_moment_data_with_kind(
        self,
        kind: nexrad_model::data::MomentDataKind,
    ) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            kind,
            self.header.number_of_data_moment_gates(),
            self.header.data_moment_range_raw(),
            self.header.data_moment_range_sample_interval_raw(),
            self.header.data_word_size(),
            self.header.scale(),
            self.header.offset(),
            self.encoded_data.into_inner().into_owned(),
        )
    }

    /// Deprecated alias that uses an unknown moment kind.
    #[cfg(feature = "nexrad-model")]
    #[deprecated(note = "use into_moment_data_with_kind for correct CFP decoding")]
    pub fn into_moment_data(self) -> nexrad_model::data::MomentData {
        self.into_moment_data_with_kind(nexrad_model::data::MomentDataKind::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::digital_radar_data::DataMomentGenericPointerType;
    use crate::messages::primitive_aliases::{Integer2, Integer4, Real4, ScaledInteger2};
    use std::borrow::Cow;

    #[test]
    fn test_decoded_values_16bit() {
        let raw_header = raw::GenericDataBlockHeader {
            reserved: Integer4::new(0),
            number_of_data_moment_gates: Integer2::new(2),
            data_moment_range: ScaledInteger2::new(0),
            data_moment_range_sample_interval: ScaledInteger2::new(1),
            tover: ScaledInteger2::new(0),
            snr_threshold: ScaledInteger2::new(0),
            control_flags: 0,
            data_word_size: 16,
            scale: Real4::new(2.0),
            offset: Real4::new(10.0),
        };

        let header = GenericDataBlockHeader::new(&raw_header);
        // Two 16-bit big-endian values: 20 and 30
        let encoded = vec![0x00, 0x14, 0x00, 0x1E];
        let block = GenericDataBlock {
            header,
            encoded_data: BinaryData::new(Cow::Owned(encoded)),
        };

        let decoded = block.decoded_values();
        assert_eq!(decoded.len(), 2);

        match decoded[0] {
            ScaledMomentValue::Value(v) => assert!((v - 5.0).abs() < 0.01),
            _ => panic!("Expected Value"),
        }
        match decoded[1] {
            ScaledMomentValue::Value(v) => assert!((v - 10.0).abs() < 0.01),
            _ => panic!("Expected Value"),
        }
    }

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
        let block = GenericDataBlock {
            header,
            encoded_data: BinaryData::new(Cow::Owned(encoded)),
        };

        let decoded = block.decoded_values_with_type(DataMomentGenericPointerType::ClutterFilterPower);
        assert_eq!(
            decoded[0],
            ScaledMomentValue::CfpStatus(CfpStatus::FilterNotApplied)
        );
        assert_eq!(
            decoded[1],
            ScaledMomentValue::CfpStatus(CfpStatus::PointClutterFilterApplied)
        );
        assert_eq!(
            decoded[2],
            ScaledMomentValue::CfpStatus(CfpStatus::DualPolOnlyFilterApplied)
        );
        assert_eq!(
            decoded[3],
            ScaledMomentValue::CfpStatus(CfpStatus::Reserved(3))
        );
        assert_eq!(
            decoded[4],
            ScaledMomentValue::CfpStatus(CfpStatus::Reserved(7))
        );
        assert_eq!(decoded[5], ScaledMomentValue::Value(8.0));
    }
}
