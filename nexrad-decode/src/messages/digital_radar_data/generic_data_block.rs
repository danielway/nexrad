use crate::binary_data::BinaryData;
use crate::messages::digital_radar_data::{GenericDataBlockHeader, ScaledMomentValue};
use crate::result::Result;
use crate::util::take_ref;
use std::fmt::Debug;

/// A generic data moment block.
#[derive(Clone, PartialEq, Debug)]
pub struct GenericDataBlock<'a> {
    /// The generic data block's header information.
    pub header: &'a GenericDataBlockHeader,

    /// The generic data block's encoded moment data.
    pub encoded_data: BinaryData<Vec<u8>>,
}

impl<'a> GenericDataBlock<'a> {
    /// Creates a new generic data moment block from the decoded header.
    pub(crate) fn parse<'b>(input: &'b mut &'a [u8]) -> Result<Self> {
        let header = take_ref::<GenericDataBlockHeader>(input)?;

        let word_size_bytes = header.data_word_size as usize / 8;
        let encoded_data_size = header.number_of_data_moment_gates.get() as usize * word_size_bytes;
        let encoded_data = BinaryData::new(vec![0; encoded_data_size]);

        Ok(Self {
            header,
            encoded_data,
        })
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
                if self.header.scale == 0.0 {
                    return ScaledMomentValue::Value(raw_value as f32);
                }

                match raw_value {
                    0 => ScaledMomentValue::BelowThreshold,
                    1 => ScaledMomentValue::RangeFolded,
                    _ => ScaledMomentValue::Value(
                        (raw_value as f32 - self.header.offset.get()) / self.header.scale.get(),
                    ),
                }
            })
            .collect()
    }

    /// Get moment data from this generic data block. Note that this will clone the underlying data.
    #[cfg(feature = "nexrad-model")]
    pub fn moment_data(&self) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            self.header.number_of_data_moment_gates.get(),
            self.header.data_moment_range.get(),
            self.header.data_moment_range_sample_interval.get(),
            self.header.scale.get(),
            self.header.offset.get(),
            self.encoded_data.0.clone(),
        )
    }

    /// Convert this generic data block into common model moment data, minimizing data copies.
    #[cfg(feature = "nexrad-model")]
    pub fn into_moment_data(self) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            self.header.number_of_data_moment_gates.get(),
            self.header.data_moment_range.get(),
            self.header.data_moment_range_sample_interval.get(),
            self.header.scale.get(),
            self.header.offset.get(),
            self.encoded_data.into_inner(),
        )
    }
}
