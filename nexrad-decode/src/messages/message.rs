use crate::messages::clutter_filter_map;
use crate::messages::digital_radar_data;
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data;
use crate::messages::volume_coverage_pattern;

/// A decoded NEXRAD Level II message with its metadata header.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    header: MessageHeader,
    contents: MessageContents,
}

impl Message {
    /// Create a new unsegmented message.
    pub(crate) fn unsegmented(header: MessageHeader, contents: MessageContents) -> Self {
        Self { header, contents }
    }

    /// This message's header.
    pub fn header(&self) -> &MessageHeader {
        &self.header
    }

    /// This message's contents.
    pub fn contents(&self) -> &MessageContents {
        &self.contents
    }

    /// Consume this message, returning ownership of its contents.
    pub fn into_contents(self) -> MessageContents {
        self.contents
    }
}

/// A decoded NEXRAD Level II message's contents.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageContents {
    /// Message type 2 "RDA Status Data" contains information about the current RDA state, system
    /// control, operating status, scanning strategy, performance parameters like transmitter power
    /// and calibration, and system alarms
    RDAStatusData(Box<rda_status_data::Message>),

    /// Message type 31 "Digital Radar Data" consists of base data information such as reflectivity,
    /// mean radial velocity, spectrum width, differential reflectivity, differential phase,
    /// correlation coefficient, azimuth angle, elevation angle, cut type, scanning strategy, and
    /// calibration parameters.
    DigitalRadarData(Box<digital_radar_data::Message>),

    /// Message type 15 "Clutter Filter Map" contains information about clutter filter maps that are
    /// used to filter clutter from radar products
    ClutterFilterMap(Box<clutter_filter_map::Message>),

    /// Message type 5 "Volume Coverage Pattern" provides details about the volume
    /// coverage pattern being used, including detailed settings for each elevation.
    VolumeCoveragePattern(Box<volume_coverage_pattern::Message>),

    Other,
}
