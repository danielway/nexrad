use crate::model::messages::clutter_filter_map;
use crate::model::messages::digital_radar_data;
use crate::model::messages::message_header::MessageHeader;
use crate::model::messages::rda_status_data;

#[derive(Debug)]
pub enum Message {
    RDAStatusData(rda_status_data::Message),
    DigitalRadarData(digital_radar_data::Message),
    ClutterFilterMap(clutter_filter_map::Message),
    Other,
}

#[derive(Debug)]
pub struct MessageWithHeader {
    pub header: MessageHeader,
    pub message: Message,
}
