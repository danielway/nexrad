use crate::messages::definitions::RedundantChannel;
use crate::messages::message_type::MessageType;
use crate::messages::primitive_aliases::{Integer1, Integer2, Integer4};
use crate::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::si::f64::Information;
#[cfg(feature = "uom")]
use uom::si::information::byte;

/// This value in the [segment_size] field of a message header indicates that the message is
/// variable-length rather than segmented.
pub const VARIABLE_LENGTH_MESSAGE_SIZE: u16 = 65535;

/// Message and system configuration information appended to the beginning of all messages.
///
/// Note that messages with a segment size of [VARIABLE_LENGTH_MESSAGE_SIZE] are not segmented and
/// instead variable-length, with the segment count and segment number positions of the header
/// (bytes 12-15) specifying the size of the full message in bytes.
#[repr(C)]
#[derive(Deserialize)]
pub struct MessageHeader {
    rpg_unknown: [u8; 12],

    /// Size of this segment in half-words. Note that this only describes this segment's size,
    /// though there could be multiple segments. In the case of a variable-length message (indicated
    /// by this field being set to [VARIABLE_LENGTH_MESSAGE_SIZE]), the full message's size is
    /// determined differently. See [message_size] and [number_of_segments] for more information.
    pub segment_size: Integer2,

    /// Whether the RDA is operating on a redundant channel.
    ///
    /// Legacy:
    ///  0 = Single Channel (no bits set)
    ///  1 = Redundant Channel 1 (bit 0 set)
    ///  2 = Redundant Channel 2 (bit 1 set)
    /// ORDA
    ///  8 = Single Channel (bit 3 set)
    ///  9 = Redundant Channel 1 (bits 0 and 3 set)
    /// 10 = Redundant Channel 2 (bits 1 and 3 set)
    pub redundant_channel: Integer1,

    /// Type discriminator.
    pub message_type: Integer1,

    /// Message sequence number.
    pub sequence_number: Integer2,

    /// This message's date represented as a count of days since 1 January 1970 00:00 GMT. It is
    /// also referred-to as a "modified Julian date" where it is the Julian date - 2440586.5.
    pub date: Integer2,

    /// Milliseconds past midnight, GMT.
    pub time: Integer4,

    /// Number of segments in this message. If the [segment_size] is less than
    /// [VARIABLE_LENGTH_MESSAGE_SIZE], this field is meaningful, otherwise bytes 12-15 (this field
    /// and [segment_number]) specify the size of the message in bytes.
    pub segment_count: Integer2,

    /// This message segment's number. If the [segment_size] is less than
    /// [VARIABLE_LENGTH_MESSAGE_SIZE], this field is meaningful, otherwise, bytes 12-15 (this field
    /// and [segment_number]) specify the size of the message in bytes.
    pub segment_number: Integer2,
}

impl MessageHeader {
    /// If this message is [segmented], this indicates this message segment's size. Otherwise, this
    /// returns [None] and [message_size] should be used to determine the message's full size.
    #[cfg(feature = "uom")]
    pub fn segment_size(&self) -> Option<Information> {
        if self.segment_size < VARIABLE_LENGTH_MESSAGE_SIZE {
            Some(Information::new::<byte>((self.segment_size * 2) as f64))
        } else {
            None
        }
    }

    /// Whether the RDA is operating on a redundant channel.
    pub fn rda_redundant_channel(&self) -> RedundantChannel {
        match self.redundant_channel {
            0 => RedundantChannel::LegacySingleChannel,
            1 => RedundantChannel::LegacyRedundantChannel1,
            2 => RedundantChannel::LegacyRedundantChannel2,
            8 => RedundantChannel::ORDASingleChannel,
            9 => RedundantChannel::ORDARedundantChannel1,
            10 => RedundantChannel::ORDARedundantChannel2,
            _ => panic!("Invalid RDA redundant channel: {}", self.redundant_channel),
        }
    }

    /// Message type discriminator.
    pub fn message_type(&self) -> MessageType {
        match self.message_type {
            1 => MessageType::RDADigitalRadarData,
            2 => MessageType::RDAStatusData,
            3 => MessageType::RDAPerformanceMaintenanceData,
            4 => MessageType::RDAConsoleMessage,
            5 => MessageType::RDAVolumeCoveragePattern,
            6 => MessageType::RDAControlCommands,
            7 => MessageType::RPGVolumeCoveragePattern,
            8 => MessageType::RPGClutterCensorZones,
            9 => MessageType::RPGRequestForData,
            10 => MessageType::RPGConsoleMessage,
            11 => MessageType::RDALoopBackTest,
            12 => MessageType::RPGLoopBackTest,
            13 => MessageType::RDAClutterFilterBypassMap,
            14 => MessageType::Spare1,
            15 => MessageType::RDAClutterFilterMap,
            16 => MessageType::ReservedFAARMSOnly1,
            17 => MessageType::ReservedFAARMSOnly2,
            18 => MessageType::RDAAdaptationData,
            20 => MessageType::Reserved1,
            21 => MessageType::Reserved2,
            22 => MessageType::Reserved3,
            23 => MessageType::Reserved4,
            24 => MessageType::ReservedFAARMSOnly3,
            25 => MessageType::ReservedFAARMSOnly4,
            26 => MessageType::ReservedFAARMSOnly5,
            29 => MessageType::Reserved5,
            31 => MessageType::RDADigitalRadarDataGenericFormat,
            32 => MessageType::RDAPRFData,
            33 => MessageType::RDALogData,
            _ => MessageType::Unknown,
        }
    }

    /// This message's date and time in UTC.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(self.date, Duration::milliseconds(self.time as i64))
    }

    /// Whether this message is segmented or variable-length. If the message is segmented, multiple
    /// message segments compose the full message. If the message is variable-length as indicated by
    /// the [segment_size] field being set to [VARIABLE_LENGTH_MESSAGE_SIZE], the full message size
    /// is determined by the [message_size] field.
    pub fn segmented(&self) -> bool {
        self.segment_size < VARIABLE_LENGTH_MESSAGE_SIZE
    }

    /// If the message is [segmented], this indicates the number of segments in the full message,
    /// otherwise this returns [None]. [message_size] can be used to determine the message's full
    /// size.
    pub fn segment_count(&self) -> Option<u16> {
        if self.segment_size < VARIABLE_LENGTH_MESSAGE_SIZE {
            Some(self.segment_count)
        } else {
            None
        }
    }

    /// If the message is [segmented], this indicates this segment's number/sequence in the message,
    /// otherwise this returns [None]. [message_size] can be used to determine the message's full
    /// size.
    pub fn segment_number(&self) -> Option<u16> {
        if self.segment_size < VARIABLE_LENGTH_MESSAGE_SIZE {
            Some(self.segment_number)
        } else {
            None
        }
    }

    /// The full size of the message in bytes. If the message is [segmented] then this is the
    /// segment size, otherwise this is the full variable-length message size.
    pub fn message_size_bytes(&self) -> u32 {
        match self.segment_count() {
            Some(_) => self.segment_size as u32 * 2,
            None => {
                let segment_number = self.segment_number as u32;
                let segment_size = self.segment_size as u32;
                (segment_number << 16) | (segment_size << 1)
            }
        }
    }

    /// The full size of the message. If the message is [segmented] then this is the segment size,
    /// otherwise this is the full variable-length message size.
    #[cfg(feature = "uom")]
    pub fn message_size(&self) -> Information {
        match self.segment_count() {
            Some(_) => {
                let segment_size_bytes = self.segment_size << 1;
                Information::new::<byte>(segment_size_bytes as f64)
            }
            None => {
                let segment_number = self.segment_number as u32;
                let segment_size = self.segment_size as u32;
                let message_size_bytes = (segment_number << 16) | segment_size;
                Information::new::<byte>(message_size_bytes as f64)
            }
        }
    }
}

#[cfg(not(feature = "uom"))]
impl Debug for MessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageHeader")
            .field("segment_size", &self.segment_size)
            .field("redundant_channel", &self.rda_redundant_channel())
            .field("message_type", &self.message_type())
            .field("sequence_number", &self.sequence_number)
            .field("date_time", &self.date_time())
            .field("segment_count", &self.segment_count())
            .field("segment_number", &self.segment_number())
            .field("message_size_bytes", &self.message_size_bytes())
            .finish()
    }
}

#[cfg(feature = "uom")]
impl Debug for MessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageHeader")
            .field("segment_size", &self.segment_size())
            .field("redundant_channel", &self.rda_redundant_channel())
            .field("message_type", &self.message_type())
            .field("sequence_number", &self.sequence_number)
            .field("date_time", &self.date_time())
            .field("segment_count", &self.segment_count())
            .field("segment_number", &self.segment_number())
            .field("message_size", &self.message_size())
            .finish()
    }
}
