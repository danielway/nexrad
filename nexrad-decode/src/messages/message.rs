use crate::messages::clutter_filter_map;
use crate::messages::digital_radar_data;
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data;

/// A decoded NEXRAD Level II message. Note that segmented messages will be represented with a
/// single [Message].
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    first_header: MessageHeader,
    subsequent_headers: Option<Vec<MessageHeader>>,
    body: MessageBody,
}

impl Message {
    /// Create a new unsegmented message.
    pub(crate) fn new(header: MessageHeader, body: MessageBody) -> Self {
        Self {
            first_header: header,
            subsequent_headers: None,
            body,
        }
    }

    /// This message's header. If segmented, this is the first segment's header.
    pub fn header(&self) -> &MessageHeader {
        &self.first_header
    }

    /// This message's headers. If unsegmented, this will simply have the [Message::header].
    pub fn headers(&self) -> Vec<&MessageHeader> {
        let mut headers = vec![&self.first_header];

        if let Some(subsequent_headers) = self.subsequent_headers.as_ref() {
            headers.extend(subsequent_headers);
        }

        headers
    }

    /// This message's contents.
    pub fn contents(&self) -> &MessageBody {
        &self.body
    }

    /// Take this message's contents, consuming the message.
    pub fn take_contents(self) -> MessageBody {
        self.body
    }
}

/// A decoded NEXRAD Level II message's contents.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageBody {
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
    ClutterFilterMap(Box<clutter_filter_map::Segment>),

    Other,
}
