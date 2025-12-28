use crate::messages::{
    clutter_filter_map, digital_radar_data, rda_status_data, volume_coverage_pattern,
};

/// A decoded NEXRAD Level II message's contents.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageContents<'a> {
    /// Message type 2 "RDA Status Data" contains information about the current RDA state, system
    /// control, operating status, scanning strategy, performance parameters like transmitter power
    /// and calibration, and system alarms
    RDAStatusData(Box<rda_status_data::Message>),

    /// Message type 31 "Digital Radar Data" consists of base data information such as reflectivity,
    /// mean radial velocity, spectrum width, differential reflectivity, differential phase,
    /// correlation coefficient, azimuth angle, elevation angle, cut type, scanning strategy, and
    /// calibration parameters.
    DigitalRadarData(Box<digital_radar_data::Message<'a>>),

    /// Message type 15 "Clutter Filter Map" contains information about clutter filter maps that are
    /// used to filter clutter from radar products
    ClutterFilterMap(Box<clutter_filter_map::Message<'a>>),

    /// Message type 5 "Volume Coverage Pattern" provides details about the volume
    /// coverage pattern being used, including detailed settings for each elevation.
    VolumeCoveragePattern(Box<volume_coverage_pattern::Message<'a>>),

    Other,
}
