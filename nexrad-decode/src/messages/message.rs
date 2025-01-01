use crate::messages::clutter_filter_map;
use crate::messages::digital_radar_data;
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data;
use crate::messages::volume_coverage_pattern;

/// A decoded NEXRAD Level II message with its metadata header.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub header: MessageHeader,
    pub contents: MessageContents,
}

/// A decoded NEXRAD Level II message's contents.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageContents {
    RDAStatusData(Box<rda_status_data::Message>),
    DigitalRadarData(Box<digital_radar_data::Message>),
    ClutterFilterMap(Box<clutter_filter_map::Message>),
    VolumeCoveragePattern(Box<volume_coverage_pattern::Message>),
    Other,
}
