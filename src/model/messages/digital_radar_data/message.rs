use crate::model::messages::digital_radar_data::{
    ElevationDataBlock, GenericDataBlock, Header, RadialDataBlock, VolumeDataBlock,
};

/// The digital radar data message includes base radar data for various products.
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
}
