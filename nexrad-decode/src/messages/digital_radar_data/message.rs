use crate::messages::digital_radar_data::{
    ElevationDataBlock, GenericDataBlock, Header, RadialDataBlock, RadialStatus, VolumeDataBlock,
};
use crate::result::{Error, Result};

/// The digital radar data message includes base radar data from a single radial for various
/// products.
#[derive(Debug)]
pub struct Message {
    /// The decoded digital radar data header.
    pub header: Header,

    /// Volume data if included in the message.
    pub volume_data_block: Option<VolumeDataBlock>,

    /// Elevation data if included in the message.
    pub elevation_data_block: Option<ElevationDataBlock>,

    /// Radial data if included in the message.
    pub radial_data_block: Option<RadialDataBlock>,

    /// Reflectivity data if included in the message.
    pub reflectivity_data_block: Option<GenericDataBlock>,

    /// Velocity data if included in the message.
    pub velocity_data_block: Option<GenericDataBlock>,

    /// Spectrum width data if included in the message.
    pub spectrum_width_data_block: Option<GenericDataBlock>,

    /// Differential reflectivity data if included in the message.
    pub differential_reflectivity_data_block: Option<GenericDataBlock>,

    /// Differential phase data if included in the message.
    pub differential_phase_data_block: Option<GenericDataBlock>,

    /// Correlation coefficient data if included in the message.
    pub correlation_coefficient_data_block: Option<GenericDataBlock>,

    /// Specific differential phase data if included in the message.
    pub specific_diff_phase_data_block: Option<GenericDataBlock>,
}

impl Message {
    /// Create a new digital radar data message with the decoded header.
    pub(crate) fn new(header: Header) -> Self {
        Self {
            header,
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
        }
    }

    /// Get a radial from this digital radar data message.
    #[cfg(feature = "nexrad-model")]
    pub fn radial(&self) -> Result<nexrad_model::data::Radial> {
        use nexrad_model::data::{Radial, RadialStatus as ModelRadialStatus};

        Ok(Radial::new(
            self.header
                .date_time()
                .ok_or(Error::MessageMissingDateError)?
                .timestamp_millis(),
            self.header.azimuth_number,
            self.header.azimuth_angle,
            self.header.azimuth_resolution_spacing as f32 * 0.5,
            match self.header.radial_status() {
                RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
                RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
                RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
                RadialStatus::VolumeScanStart => ModelRadialStatus::VolumeScanStart,
                RadialStatus::VolumeScanEnd => ModelRadialStatus::VolumeScanEnd,
                RadialStatus::ElevationStartVCPFinal => ModelRadialStatus::ElevationStartVCPFinal,
            },
            self.header.elevation_angle,
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
    pub fn into_radial(self) -> Result<nexrad_model::data::Radial> {
        use nexrad_model::data::{Radial, RadialStatus as ModelRadialStatus};

        Ok(Radial::new(
            self.header
                .date_time()
                .ok_or(Error::MessageMissingDateError)?
                .timestamp_millis(),
            self.header.azimuth_number,
            self.header.azimuth_angle,
            self.header.azimuth_resolution_spacing as f32 * 0.5,
            match self.header.radial_status() {
                RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
                RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
                RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
                RadialStatus::VolumeScanStart => ModelRadialStatus::VolumeScanStart,
                RadialStatus::VolumeScanEnd => ModelRadialStatus::VolumeScanEnd,
                RadialStatus::ElevationStartVCPFinal => ModelRadialStatus::ElevationStartVCPFinal,
            },
            self.header.elevation_angle,
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
