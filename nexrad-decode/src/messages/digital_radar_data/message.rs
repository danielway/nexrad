use super::raw;
use super::{
    DataBlock, DataBlockId, DataBlockName, ElevationDataBlock, GenericDataBlock, Header,
    RadialDataBlock, VolumeDataBlock,
};
use crate::result::{Error, Result};
use crate::slice_reader::SliceReader;

/// The digital radar data message includes base radar data from a single radial for various
/// products.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    /// The decoded digital radar data header.
    header: Header<'a>,

    /// Volume data if included in the message.
    volume_data_block: Option<DataBlock<'a, VolumeDataBlock<'a>>>,

    /// Elevation data if included in the message.
    elevation_data_block: Option<DataBlock<'a, ElevationDataBlock<'a>>>,

    /// Radial data if included in the message.
    radial_data_block: Option<DataBlock<'a, RadialDataBlock<'a>>>,

    /// Reflectivity data if included in the message.
    reflectivity_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,

    /// Velocity data if included in the message.
    velocity_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,

    /// Spectrum width data if included in the message.
    spectrum_width_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,

    /// Differential reflectivity data if included in the message.
    differential_reflectivity_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,

    /// Differential phase data if included in the message.
    differential_phase_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,

    /// Correlation coefficient data if included in the message.
    correlation_coefficient_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,

    /// Clutter filter power (CFP) data if included in the message.
    clutter_filter_power_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,
}

impl<'a> Message<'a> {
    pub(crate) fn parse(reader: &mut SliceReader<'a>) -> Result<Self> {
        let start_position = reader.position();
        let raw_header = reader.take_ref::<raw::Header>()?;

        let pointers_space = raw_header.data_block_count.get() as usize * size_of::<u32>();
        let pointers_raw = reader.take_bytes(pointers_space)?;

        let pointers = pointers_raw
            .as_chunks::<{ size_of::<u32>() }>()
            .0
            .iter()
            .copied()
            .map(u32::from_be_bytes)
            .collect::<Vec<_>>();

        let mut message = Self {
            header: Header::new(raw_header),
            volume_data_block: None,
            elevation_data_block: None,
            radial_data_block: None,
            reflectivity_data_block: None,
            velocity_data_block: None,
            spectrum_width_data_block: None,
            differential_reflectivity_data_block: None,
            differential_phase_data_block: None,
            correlation_coefficient_data_block: None,
            clutter_filter_power_data_block: None,
        };

        for pointer in pointers {
            let relative_position = reader.position() - start_position;
            let pointer_position = pointer as usize;

            if relative_position < pointer_position {
                reader.advance(pointer_position - relative_position);
            } else if relative_position > pointer_position {
                return Err(Error::InvalidDataBlockPointer {
                    bytes: relative_position - pointer_position,
                    position: reader.position(),
                });
            }

            let block_id = reader.take_ref::<raw::DataBlockId>()?;
            let id = DataBlockId::new(block_id);

            match id.data_block_name() {
                DataBlockName::Volume => {
                    // VOL expanded from 44 to 52 bytes at Build 20.0 (ICD 2620002U).
                    if reader.peek_u16().is_some_and(|lrtup| lrtup <= 44) {
                        let block = reader.take_ref::<raw::VolumeDataBlockLegacy>()?;
                        message.volume_data_block =
                            Some(DataBlock::new(id, VolumeDataBlock::new_legacy(block)));
                    } else {
                        let block = reader.take_ref::<raw::VolumeDataBlock>()?;
                        message.volume_data_block =
                            Some(DataBlock::new(id, VolumeDataBlock::new(block)));
                    }
                }
                DataBlockName::Elevation => {
                    let elevation_block = reader.take_ref::<raw::ElevationDataBlock>()?;
                    message.elevation_data_block =
                        Some(DataBlock::new(id, ElevationDataBlock::new(elevation_block)));
                }
                DataBlockName::Radial => {
                    // RAD expanded from 20 to 28 bytes at Build 12.0 (ICD 2620002K).
                    if reader.peek_u16().is_some_and(|lrtup| lrtup <= 20) {
                        let block = reader.take_ref::<raw::RadialDataBlockLegacy>()?;
                        message.radial_data_block =
                            Some(DataBlock::new(id, RadialDataBlock::new_legacy(block)));
                    } else {
                        let block = reader.take_ref::<raw::RadialDataBlock>()?;
                        message.radial_data_block =
                            Some(DataBlock::new(id, RadialDataBlock::new(block)));
                    }
                }
                name => {
                    let generic_block = GenericDataBlock::parse(reader)?;
                    match name {
                        DataBlockName::Reflectivity => {
                            message.reflectivity_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        DataBlockName::Velocity => {
                            message.velocity_data_block = Some(DataBlock::new(id, generic_block));
                        }
                        DataBlockName::SpectrumWidth => {
                            message.spectrum_width_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        DataBlockName::DifferentialReflectivity => {
                            message.differential_reflectivity_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        DataBlockName::DifferentialPhase => {
                            message.differential_phase_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        DataBlockName::CorrelationCoefficient => {
                            message.correlation_coefficient_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        DataBlockName::ClutterFilterPower => {
                            message.clutter_filter_power_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        _ => {
                            // Unknown block type - skip for forward compatibility with newer formats
                            log::warn!("Skipping unknown generic data block type: {:?}", block_id);
                        }
                    }
                }
            }
        }

        Ok(message)
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: self.header.into_owned(),
            volume_data_block: self
                .volume_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            elevation_data_block: self
                .elevation_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            radial_data_block: self
                .radial_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            reflectivity_data_block: self
                .reflectivity_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            velocity_data_block: self
                .velocity_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            spectrum_width_data_block: self
                .spectrum_width_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            differential_reflectivity_data_block: self
                .differential_reflectivity_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            differential_phase_data_block: self
                .differential_phase_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            correlation_coefficient_data_block: self
                .correlation_coefficient_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
            clutter_filter_power_data_block: self
                .clutter_filter_power_data_block
                .map(|b| b.into_owned_with(|inner| inner.into_owned())),
        }
    }

    /// The decoded digital radar data header.
    pub fn header(&self) -> &Header<'a> {
        &self.header
    }

    /// Volume data if included in the message.
    pub fn volume_data_block(&self) -> Option<&DataBlock<'a, VolumeDataBlock<'a>>> {
        self.volume_data_block.as_ref()
    }

    /// Elevation data if included in the message.
    pub fn elevation_data_block(&self) -> Option<&DataBlock<'a, ElevationDataBlock<'a>>> {
        self.elevation_data_block.as_ref()
    }

    /// Radial data if included in the message.
    pub fn radial_data_block(&self) -> Option<&DataBlock<'a, RadialDataBlock<'a>>> {
        self.radial_data_block.as_ref()
    }

    /// Reflectivity data if included in the message.
    pub fn reflectivity_data_block(&self) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.reflectivity_data_block.as_ref()
    }

    /// Velocity data if included in the message.
    pub fn velocity_data_block(&self) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.velocity_data_block.as_ref()
    }

    /// Spectrum width data if included in the message.
    pub fn spectrum_width_data_block(&self) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.spectrum_width_data_block.as_ref()
    }

    /// Differential reflectivity data if included in the message.
    pub fn differential_reflectivity_data_block(
        &self,
    ) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.differential_reflectivity_data_block.as_ref()
    }

    /// Differential phase data if included in the message.
    pub fn differential_phase_data_block(&self) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.differential_phase_data_block.as_ref()
    }

    /// Correlation coefficient data if included in the message.
    pub fn correlation_coefficient_data_block(
        &self,
    ) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.correlation_coefficient_data_block.as_ref()
    }

    /// Clutter filter power (CFP) data if included in the message.
    /// CFP represents the difference between clutter-filtered and unfiltered reflectivity.
    pub fn clutter_filter_power_data_block(&self) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.clutter_filter_power_data_block.as_ref()
    }

    /// Get a radial from this digital radar data message. This clones the underlying moment
    /// data; use [`into_radial`](Self::into_radial) to avoid the copy when the message is no
    /// longer needed.
    #[cfg(feature = "model")]
    pub fn radial(&self) -> crate::result::Result<nexrad_model::data::Radial> {
        let moments = [
            &self.reflectivity_data_block,
            &self.velocity_data_block,
            &self.spectrum_width_data_block,
            &self.differential_reflectivity_data_block,
            &self.differential_phase_data_block,
            &self.correlation_coefficient_data_block,
            &self.clutter_filter_power_data_block,
        ]
        .into_iter()
        .flatten()
        .map(DataBlock::moment_data)
        .collect::<crate::result::Result<Vec<_>>>()?;

        Self::build_radial(&self.header, moments)
    }

    /// Convert this digital radar data message into a common model radial, minimizing data copy.
    #[cfg(feature = "model")]
    pub fn into_radial(self) -> crate::result::Result<nexrad_model::data::Radial> {
        let moments = [
            self.reflectivity_data_block,
            self.velocity_data_block,
            self.spectrum_width_data_block,
            self.differential_reflectivity_data_block,
            self.differential_phase_data_block,
            self.correlation_coefficient_data_block,
            self.clutter_filter_power_data_block,
        ]
        .into_iter()
        .flatten()
        .map(DataBlock::into_moment_data)
        .collect::<crate::result::Result<Vec<_>>>()?;

        Self::build_radial(&self.header, moments)
    }

    #[cfg(feature = "model")]
    fn build_radial(
        header: &Header<'_>,
        moments: Vec<nexrad_model::data::MomentData>,
    ) -> crate::result::Result<nexrad_model::data::Radial> {
        use crate::result::Error;
        use nexrad_model::data::Radial;

        let mut builder = Radial::builder(
            header.azimuth_number(),
            header.azimuth_angle_degrees(),
            header.elevation_number(),
            header.elevation_angle_degrees(),
        )
        .collection_timestamp(
            header
                .date_time()
                .ok_or(Error::MessageMissingDate)?
                .timestamp_millis(),
        )
        .azimuth_spacing_degrees(header.azimuth_resolution_spacing_degrees())
        .radial_status(header.radial_status().into());

        for moment in moments {
            builder = builder.moment(moment);
        }

        Ok(builder.build())
    }
}

#[cfg(feature = "model")]
impl Message<'_> {
    /// The radar site that produced this message, built from the header's radar identifier and
    /// the volume data block's location and heights. Returns `None` when the message carries no
    /// volume data block.
    pub fn site(&self) -> Option<nexrad_model::meta::Site> {
        let volume = self.volume_data_block.as_ref()?.inner();
        Some(
            nexrad_model::meta::Site::builder(
                *self.header.radar_identifier_raw(),
                volume.latitude_degrees() as f64,
                volume.longitude_degrees() as f64,
            )
            .height_meters(volume.site_height_meters())
            .tower_height_meters(volume.tower_height_meters())
            .build(),
        )
    }
}
