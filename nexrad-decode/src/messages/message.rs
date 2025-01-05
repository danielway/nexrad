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
        Self {
            header,
            contents,
        }
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
    RDAStatusData(Box<rda_status_data::Message>),
    DigitalRadarData(Box<digital_radar_data::Message>),
    ClutterFilterMap(Box<clutter_filter_map::Message>),
    VolumeCoveragePattern(Box<volume_coverage_pattern::Message>),
    Other,
}
