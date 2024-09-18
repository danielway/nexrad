use crate::messages::clutter_filter_map;
use crate::messages::digital_radar_data;
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data;

/// A decoded NEXRAD Level II message with its metadata header.
#[derive(Debug)]
pub struct MessageWithHeader {
    pub header: MessageHeader,
    pub message: Message,
}

/// A decoded NEXRAD Level II message.
#[derive(Debug)]
pub enum Message {
    RDAStatusData(Box<rda_status_data::Message>),
    DigitalRadarData(Box<digital_radar_data::Message>),
    ClutterFilterMap(Box<clutter_filter_map::Message>),
    Other,
}
