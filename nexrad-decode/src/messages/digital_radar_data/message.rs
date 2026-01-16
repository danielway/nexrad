use super::raw;
use super::{
    DataBlock, DataBlockId, ElevationDataBlock, GenericDataBlock, Header, RadialDataBlock,
    RadialStatus, VolumeDataBlock,
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

    /// Specific differential phase data if included in the message.
    specific_diff_phase_data_block: Option<DataBlock<'a, GenericDataBlock<'a>>>,
}

impl<'a> Message<'a> {
    pub(crate) fn parse(reader: &mut SliceReader<'a>) -> Result<Self> {
        let start_position = reader.position();
        let raw_header = reader.take_ref::<raw::Header>()?;

        let pointers_space = raw_header.data_block_count.get() as usize * size_of::<u32>();
        let pointers_raw = reader.take_bytes(pointers_space)?;

        let pointers = pointers_raw
            .chunks_exact(size_of::<u32>())
            .map(|v| {
                v.try_into()
                    .map_err(|_| Error::DecodingError("message pointers".to_string()))
                    .map(u32::from_be_bytes)
            })
            .collect::<Result<Vec<_>>>()?;

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
            specific_diff_phase_data_block: None,
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

            match &block_id.data_name {
                b"VOL" => {
                    // Determine which format to parse. First check build number if available,
                    // otherwise peek at lrtup field to detect format.
                    // Legacy builds (19.0 and earlier) use 40-byte VolumeDataBlock,
                    // modern builds (20.0+) use 48-byte VolumeDataBlock.
                    let use_legacy = if let Some(build) = reader.build_number() {
                        build.uses_legacy_volume_data_block()
                    } else {
                        // Fall back to lrtup detection: peek at first 2 bytes to get block size.
                        // lrtup includes the DataBlockId (4 bytes), so:
                        // - Legacy format (40-byte struct): lrtup = 44
                        // - Modern format (48-byte struct): lrtup = 52
                        let remaining = reader.remaining();
                        if remaining.len() >= 2 {
                            let lrtup = u16::from_be_bytes([remaining[0], remaining[1]]);
                            lrtup <= 44
                        } else {
                            false // Default to modern if we can't peek
                        }
                    };

                    if use_legacy {
                        let volume_block = reader.take_ref::<raw::VolumeDataBlockLegacy>()?;
                        message.volume_data_block = Some(DataBlock::new(
                            id,
                            VolumeDataBlock::new_legacy(volume_block),
                        ));
                    } else {
                        let volume_block = reader.take_ref::<raw::VolumeDataBlock>()?;
                        message.volume_data_block =
                            Some(DataBlock::new(id, VolumeDataBlock::new(volume_block)));
                    }
                }
                b"ELV" => {
                    let elevation_block = reader.take_ref::<raw::ElevationDataBlock>()?;
                    message.elevation_data_block =
                        Some(DataBlock::new(id, ElevationDataBlock::new(elevation_block)));
                }
                b"RAD" => {
                    let radial_block = reader.take_ref::<raw::RadialDataBlock>()?;
                    message.radial_data_block =
                        Some(DataBlock::new(id, RadialDataBlock::new(radial_block)));
                }
                _ => {
                    let generic_block = GenericDataBlock::parse(reader)?;
                    match &block_id.data_name {
                        b"REF" => {
                            message.reflectivity_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        b"VEL" => {
                            message.velocity_data_block = Some(DataBlock::new(id, generic_block));
                        }
                        b"SW " => {
                            message.spectrum_width_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        b"ZDR" => {
                            message.differential_reflectivity_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        b"PHI" => {
                            message.differential_phase_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        b"RHO" => {
                            message.correlation_coefficient_data_block =
                                Some(DataBlock::new(id, generic_block));
                        }
                        b"CFP" => {
                            message.specific_diff_phase_data_block =
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
            specific_diff_phase_data_block: self
                .specific_diff_phase_data_block
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

    /// Specific differential phase data if included in the message.
    pub fn specific_diff_phase_data_block(&self) -> Option<&DataBlock<'a, GenericDataBlock<'a>>> {
        self.specific_diff_phase_data_block.as_ref()
    }

    /// Get a radial from this digital radar data message.
    #[cfg(feature = "nexrad-model")]
    pub fn radial(&self) -> crate::result::Result<nexrad_model::data::Radial> {
        use crate::result::Error;
        use nexrad_model::data::{Radial, RadialStatus as ModelRadialStatus};

        Ok(Radial::new(
            self.header
                .date_time()
                .ok_or(Error::MessageMissingDateError)?
                .timestamp_millis(),
            self.header.azimuth_number(),
            self.header.azimuth_angle_raw(),
            self.header.azimuth_resolution_spacing_raw() as f32 * 0.5,
            match self.header.radial_status() {
                RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
                RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
                RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
                RadialStatus::VolumeScanStart => ModelRadialStatus::VolumeScanStart,
                RadialStatus::VolumeScanEnd => ModelRadialStatus::VolumeScanEnd,
                RadialStatus::ElevationStartVCPFinal => ModelRadialStatus::ElevationStartVCPFinal,
            },
            self.header.elevation_number(),
            self.header.elevation_angle_raw(),
            self.reflectivity_data_block
                .as_ref()
                .map(|block| block.moment_data()),
            self.velocity_data_block
                .as_ref()
                .map(|block| block.moment_data()),
            self.spectrum_width_data_block
                .as_ref()
                .map(|block| block.moment_data()),
            self.differential_reflectivity_data_block
                .as_ref()
                .map(|block| block.moment_data()),
            self.differential_phase_data_block
                .as_ref()
                .map(|block| block.moment_data()),
            self.correlation_coefficient_data_block
                .as_ref()
                .map(|block| block.moment_data()),
            self.specific_diff_phase_data_block
                .as_ref()
                .map(|block| block.moment_data()),
        ))
    }

    /// Convert this digital radar data message into a common model radial, minimizing data copy.
    #[cfg(feature = "nexrad-model")]
    pub fn into_radial(self) -> crate::result::Result<nexrad_model::data::Radial> {
        use crate::result::Error;
        use nexrad_model::data::{Radial, RadialStatus as ModelRadialStatus};

        Ok(Radial::new(
            self.header
                .date_time()
                .ok_or(Error::MessageMissingDateError)?
                .timestamp_millis(),
            self.header.azimuth_number(),
            self.header.azimuth_angle_raw(),
            self.header.azimuth_resolution_spacing_raw() as f32 * 0.5,
            match self.header.radial_status() {
                RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
                RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
                RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
                RadialStatus::VolumeScanStart => ModelRadialStatus::VolumeScanStart,
                RadialStatus::VolumeScanEnd => ModelRadialStatus::VolumeScanEnd,
                RadialStatus::ElevationStartVCPFinal => ModelRadialStatus::ElevationStartVCPFinal,
            },
            self.header.elevation_number(),
            self.header.elevation_angle_raw(),
            self.reflectivity_data_block
                .map(|block| block.into_inner().into_moment_data()),
            self.velocity_data_block
                .map(|block| block.into_inner().into_moment_data()),
            self.spectrum_width_data_block
                .map(|block| block.into_inner().into_moment_data()),
            self.differential_reflectivity_data_block
                .map(|block| block.into_inner().into_moment_data()),
            self.differential_phase_data_block
                .map(|block| block.into_inner().into_moment_data()),
            self.correlation_coefficient_data_block
                .map(|block| block.into_inner().into_moment_data()),
            self.specific_diff_phase_data_block
                .map(|block| block.into_inner().into_moment_data()),
        ))
    }
}
