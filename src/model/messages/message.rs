use crate::model::messages::message_header::MessageHeader;
use crate::model::messages::rda_status_data;

#[derive(Debug)]
pub enum Message {
    RDAStatusData(rda_status_data::Message),
    Other,
}

#[derive(Debug)]
pub struct MessageWithHeader {
    pub header: MessageHeader,
    pub message: Message,
}
