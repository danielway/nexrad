use super::primitive_aliases::{Integer1, Integer2, Integer4};
use crate::binary_data::BinaryData;
use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// Sentinel value for the `segment_size` field indicating variable-length framing.
pub(crate) const VARIABLE_LENGTH_MESSAGE_SIZE: u16 = 65535;

/// Message and system configuration information appended to the beginning of all messages, read
/// directly from the wire.
///
/// The public view of this structure is [`crate::messages::MessageHeader`].
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, FromBytes, Immutable, KnownLayout)]
pub(crate) struct MessageHeader {
    /// Unknown/reserved bytes from RPG.
    pub(crate) rpg_unknown: BinaryData<[u8; 12]>,

    /// Size of this segment in half-words. Note that this only describes this segment's size,
    /// though there could be multiple segments. In the case of a variable-length message
    /// (indicated by this field being set to 65535/0xFFFF), the full message's size is determined
    /// differently.
    pub(crate) segment_size: Integer2,

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
    pub(crate) redundant_channel: Integer1,

    /// Type discriminator.
    pub(crate) message_type: Integer1,

    /// Message sequence number.
    pub(crate) sequence_number: Integer2,

    /// This message's date represented as a count of days since 1 January 1970 00:00 GMT. It is
    /// also referred-to as a "modified Julian date" where it is the Julian date - 2440586.5.
    pub(crate) date: Integer2,

    /// Milliseconds past midnight, GMT.
    pub(crate) time: Integer4,

    /// Number of segments in this message. If the message is segmented, this field is meaningful,
    /// otherwise bytes 12-15 (this field and `segment_number`) specify the size of the message in
    /// bytes.
    pub(crate) segment_count: Integer2,

    /// This message segment's number. If the message is segmented, this field is meaningful,
    /// otherwise bytes 12-15 (this field and `segment_count`) specify the size of the message in
    /// bytes.
    pub(crate) segment_number: Integer2,
}
