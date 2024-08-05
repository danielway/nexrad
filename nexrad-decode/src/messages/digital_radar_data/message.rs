use crate::messages::digital_radar_data::{
    ElevationDataBlock, GenericDataBlock, Header, RadialDataBlock, VolumeDataBlock,
};

#[cfg(feature = "nexrad-model")]
use nexrad_model::data::{Radial, RadialStatus};
use nexrad_model::meta::Site;

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

    /// The radar site's metadata if included on this message.
    #[cfg(feature = "nexrad-model")]
    pub fn site_metadata(&self) -> Option<Site> {
        self.volume_data_block.as_ref().map(|block| {
            Site::new(
                self.header.radar_identifier,
                block.latitude,
                block.longitude,
                block.site_height,
                block.feedhorn_height,
            )
        })
    }

    /// Maps this message into a common model radial.
    #[cfg(feature = "nexrad-model")]
    pub fn radial(&self) -> Radial {
        const MILLIS_PER_DAY: i64 = 86_400_000;
        Radial::new(
            self.header.date as i64 * MILLIS_PER_DAY + self.header.time as i64,
            self.header.azimuth_number,
            self.header.azimuth_angle,
            match self.header.azimuth_resolution_spacing {
                1 => 0.5,
                _ => 1.0,
            },
            match self.header.radial_status {
                0 => RadialStatus::ElevationStart,
                1 => RadialStatus::IntermediateRadialData,
                2 => RadialStatus::ElevationEnd,
                3 => RadialStatus::VolumeScanStart,
                4 => RadialStatus::VolumeScanEnd,
                _ => RadialStatus::ElevationStartVCPFinal,
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
        )
    }
}
