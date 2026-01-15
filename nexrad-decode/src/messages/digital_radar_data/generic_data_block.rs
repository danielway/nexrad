use super::raw;
use super::{GenericDataBlockHeader, ScaledMomentValue};
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
        self.encoded_data
            .iter()
            .copied()
            .map(|raw_value| {
                if self.header.scale() == 0.0 {
                    return ScaledMomentValue::Value(raw_value as f32);
                }

                match raw_value {
                    0 => ScaledMomentValue::BelowThreshold,
                    1 => ScaledMomentValue::RangeFolded,
                    _ => ScaledMomentValue::Value(
                        (raw_value as f32 - self.header.offset()) / self.header.scale(),
                    ),
                }
            })
            .collect()
    }

    /// Get moment data from this generic data block. Note that this will clone the underlying data.
    #[cfg(feature = "nexrad-model")]
    pub fn moment_data(&self) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            self.header.number_of_data_moment_gates(),
            self.header.data_moment_range_raw(),
            self.header.data_moment_range_sample_interval_raw(),
            self.header.scale(),
            self.header.offset(),
            self.encoded_data.to_vec(),
        )
    }

    /// Convert this generic data block into common model moment data.
    #[cfg(feature = "nexrad-model")]
    pub fn into_moment_data(self) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            self.header.number_of_data_moment_gates(),
            self.header.data_moment_range_raw(),
            self.header.data_moment_range_sample_interval_raw(),
            self.header.scale(),
            self.header.offset(),
            self.encoded_data.into_inner().into_owned(),
        )
    }
}
