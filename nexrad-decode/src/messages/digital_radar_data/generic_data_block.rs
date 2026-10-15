use super::raw;
use super::GenericDataBlockHeader;
use crate::binary_data::BinaryData;
use crate::result::Result;
use crate::slice_reader::SliceReader;
use std::borrow::Cow;

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

    /// Raw gate values for this moment/radial ordered in ascending distance from the radar. These
    /// values are stored in a fixed-point representation using the header's offset and scale fields.
    /// With the `model` feature, use `DataBlock::moment_data` on the enclosing block to decode them.
    pub fn encoded_values(&self) -> &[u8] {
        &self.encoded_data
    }
}

#[cfg(feature = "model")]
impl<'a> super::DataBlock<'a, GenericDataBlock<'a>> {
    /// The model product for this block, derived from its data block name.
    fn product(&self) -> Result<nexrad_model::data::Product> {
        self.id().data_block_name().product().ok_or_else(|| {
            crate::result::Error::UnknownDataBlockType {
                block_type: *self.id().data_name_raw(),
                offset: self.offset(),
            }
        })
    }

    /// Create a [`MomentData`](nexrad_model::data::MomentData) from this generic data block,
    /// cloning the underlying encoded data. The moment's product is derived from the block's
    /// data name.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownDataBlockType`](crate::result::Error::UnknownDataBlockType) if the
    /// block's data name is not one of the known moment names (`REF`, `VEL`, `SW `, `ZDR`,
    /// `PHI`, `RHO`, `CFP`), or [`Error::Model`](crate::result::Error::Model) if the block's
    /// encoding parameters are inconsistent with its data.
    pub fn moment_data(&self) -> Result<nexrad_model::data::MomentData> {
        let header = self.inner().header();
        Ok(nexrad_model::data::MomentData::from_fixed_point(
            self.product()?,
            usize::from(header.number_of_data_moment_gates()),
            header.data_moment_range_raw(),
            header.data_moment_range_sample_interval_raw(),
            header.data_word_size_raw(),
            header.scale(),
            header.offset(),
            self.inner().encoded_values().to_vec(),
        )?)
    }

    /// Convert this generic data block into a [`MomentData`](nexrad_model::data::MomentData),
    /// consuming the encoded data without copying. The moment's product is derived from the
    /// block's data name.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownDataBlockType`](crate::result::Error::UnknownDataBlockType) if the
    /// block's data name is not one of the known moment names (`REF`, `VEL`, `SW `, `ZDR`,
    /// `PHI`, `RHO`, `CFP`), or [`Error::Model`](crate::result::Error::Model) if the block's
    /// encoding parameters are inconsistent with its data.
    pub fn into_moment_data(self) -> Result<nexrad_model::data::MomentData> {
        let product = self.product()?;
        let block = self.into_inner();
        Ok(nexrad_model::data::MomentData::from_fixed_point(
            product,
            usize::from(block.header.number_of_data_moment_gates()),
            block.header.data_moment_range_raw(),
            block.header.data_moment_range_sample_interval_raw(),
            block.header.data_word_size_raw(),
            block.header.scale(),
            block.header.offset(),
            block.encoded_data.into_inner().into_owned(),
        )?)
    }
}
