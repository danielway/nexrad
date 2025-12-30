use crate::messages::digital_radar_data::{
    DataBlockId, ElevationDataBlock, GenericDataBlock, Header, RadialDataBlock, VolumeDataBlock,
};
use crate::result::{Error, Result};
use crate::slice_reader::SliceReader;
use std::borrow::Cow;

/// The digital radar data message includes base radar data from a single radial for various
/// products.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    /// The decoded digital radar data header.
    pub header: Cow<'a, Header>,

    /// Volume data if included in the message.
    pub volume_data_block: Option<Cow<'a, VolumeDataBlock>>,

    /// Elevation data if included in the message.
    pub elevation_data_block: Option<Cow<'a, ElevationDataBlock>>,

    /// Radial data if included in the message.
    pub radial_data_block: Option<Cow<'a, RadialDataBlock>>,

    /// Reflectivity data if included in the message.
    pub reflectivity_data_block: Option<GenericDataBlock<'a>>,

    /// Velocity data if included in the message.
    pub velocity_data_block: Option<GenericDataBlock<'a>>,

    /// Spectrum width data if included in the message.
    pub spectrum_width_data_block: Option<GenericDataBlock<'a>>,

    /// Differential reflectivity data if included in the message.
    pub differential_reflectivity_data_block: Option<GenericDataBlock<'a>>,

    /// Differential phase data if included in the message.
    pub differential_phase_data_block: Option<GenericDataBlock<'a>>,

    /// Correlation coefficient data if included in the message.
    pub correlation_coefficient_data_block: Option<GenericDataBlock<'a>>,

    /// Specific differential phase data if included in the message.
    pub specific_diff_phase_data_block: Option<GenericDataBlock<'a>>,
}

impl<'a> Message<'a> {
    pub(crate) fn parse(reader: &mut SliceReader<'a>) -> Result<Self> {
        let start_position = reader.position();
        let header = reader.take_ref::<Header>()?;

        let pointers_space = header.data_block_count.get() as usize * size_of::<u32>();
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
            header: Cow::Borrowed(header),
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
                panic!(
                    "invalid pointer, cannot rewind {} bytes",
                    relative_position - pointer_position
                );
            }

            let block_id = reader.take_ref::<DataBlockId>()?;

            match &block_id.data_name {
                b"VOL" => {
                    let volume_block = reader.take_ref::<VolumeDataBlock>()?;
                    message.volume_data_block = Some(Cow::Borrowed(volume_block));
                }
                b"ELV" => {
                    let elevation_block = reader.take_ref::<ElevationDataBlock>()?;
                    message.elevation_data_block = Some(Cow::Borrowed(elevation_block));
                }
                b"RAD" => {
                    let radial_block = reader.take_ref::<RadialDataBlock>()?;
                    message.radial_data_block = Some(Cow::Borrowed(radial_block));
                }
                _ => {
                    let generic_block = GenericDataBlock::parse(reader)?;
                    match &block_id.data_name {
                        b"REF" => {
                            message.reflectivity_data_block = Some(generic_block);
                        }
                        b"VEL" => {
                            message.velocity_data_block = Some(generic_block);
                        }
                        b"SW " => {
                            message.spectrum_width_data_block = Some(generic_block);
                        }
                        b"ZDR" => {
                            message.differential_reflectivity_data_block = Some(generic_block);
                        }
                        b"PHI" => {
                            message.differential_phase_data_block = Some(generic_block);
                        }
                        b"RHO" => {
                            message.correlation_coefficient_data_block = Some(generic_block);
                        }
                        b"CFP" => {
                            message.specific_diff_phase_data_block = Some(generic_block);
                        }
                        _ => panic!("Unknown generic data block type: {block_id:?}"),
                    }
                }
            }
        }

        Ok(message)
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: Cow::Owned(self.header.into_owned()),
            volume_data_block: self.volume_data_block.map(|b| Cow::Owned(b.into_owned())),
            elevation_data_block: self
                .elevation_data_block
                .map(|b| Cow::Owned(b.into_owned())),
            radial_data_block: self.radial_data_block.map(|b| Cow::Owned(b.into_owned())),
            reflectivity_data_block: self.reflectivity_data_block.map(|b| b.into_owned()),
            velocity_data_block: self.velocity_data_block.map(|b| b.into_owned()),
            spectrum_width_data_block: self.spectrum_width_data_block.map(|b| b.into_owned()),
            differential_reflectivity_data_block: self
                .differential_reflectivity_data_block
                .map(|b| b.into_owned()),
            differential_phase_data_block: self
                .differential_phase_data_block
                .map(|b| b.into_owned()),
            correlation_coefficient_data_block: self
                .correlation_coefficient_data_block
                .map(|b| b.into_owned()),
            specific_diff_phase_data_block: self
                .specific_diff_phase_data_block
                .map(|b| b.into_owned()),
        }
    }

    /// Get a radial from this digital radar data message.
    #[cfg(feature = "nexrad-model")]
    pub fn radial(&self) -> crate::result::Result<nexrad_model::data::Radial> {
        use crate::messages::digital_radar_data::RadialStatus;
        use crate::result::Error;
        use nexrad_model::data::{Radial, RadialStatus as ModelRadialStatus};

        Ok(Radial::new(
            self.header
                .date_time()
                .ok_or(Error::MessageMissingDateError)?
                .timestamp_millis(),
            self.header.azimuth_number.get(),
            self.header.azimuth_angle.get(),
            self.header.azimuth_resolution_spacing as f32 * 0.5,
            match self.header.radial_status() {
                RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
                RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
                RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
                RadialStatus::VolumeScanStart => ModelRadialStatus::VolumeScanStart,
                RadialStatus::VolumeScanEnd => ModelRadialStatus::VolumeScanEnd,
                RadialStatus::ElevationStartVCPFinal => ModelRadialStatus::ElevationStartVCPFinal,
            },
            self.header.elevation_number,
            self.header.elevation_angle.get(),
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
        use crate::messages::digital_radar_data::RadialStatus;
        use crate::result::Error;
        use nexrad_model::data::{Radial, RadialStatus as ModelRadialStatus};

        Ok(Radial::new(
            self.header
                .date_time()
                .ok_or(Error::MessageMissingDateError)?
                .timestamp_millis(),
            self.header.azimuth_number.get(),
            self.header.azimuth_angle.get(),
            self.header.azimuth_resolution_spacing as f32 * 0.5,
            match self.header.radial_status() {
                RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
                RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
                RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
                RadialStatus::VolumeScanStart => ModelRadialStatus::VolumeScanStart,
                RadialStatus::VolumeScanEnd => ModelRadialStatus::VolumeScanEnd,
                RadialStatus::ElevationStartVCPFinal => ModelRadialStatus::ElevationStartVCPFinal,
            },
            self.header.elevation_number,
            self.header.elevation_angle.get(),
            self.reflectivity_data_block
                .map(|block| block.into_moment_data()),
            self.velocity_data_block
                .map(|block| block.into_moment_data()),
            self.spectrum_width_data_block
                .map(|block| block.into_moment_data()),
            self.differential_reflectivity_data_block
                .map(|block| block.into_moment_data()),
            self.differential_phase_data_block
                .map(|block| block.into_moment_data()),
            self.correlation_coefficient_data_block
                .map(|block| block.into_moment_data()),
            self.specific_diff_phase_data_block
                .map(|block| block.into_moment_data()),
        ))
    }
}
