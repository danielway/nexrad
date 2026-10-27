use crate::messages::raw::{self, VARIABLE_LENGTH_MESSAGE_SIZE};
use crate::messages::{MessageType, RedundantChannel};
use crate::result::{Error, Result};
use crate::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use zerocopy::FromBytes;


/// Message and system configuration information appended to the beginning of all messages.
///
/// Use [`is_fixed_frame()`](Self::is_fixed_frame) to determine the framing mode, and
/// [`message_size_bytes()`](Self::message_size_bytes) for the message size.
///
/// This header's contents correspond to ICD 2620002AA section 3.2.4.1 Table II.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MessageHeader<'a> {
    raw: Cow<'a, raw::MessageHeader>,
}

impl Debug for MessageHeader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<'a> MessageHeader<'a> {
    /// The size of a message header on the wire, in bytes.
    pub const SIZE: usize = 28;

    /// Parse a message header from the first [`SIZE`](Self::SIZE) bytes of `bytes`, borrowing
    /// from the input. Any bytes after the header are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnexpectedEof`] with offset 0 if `bytes` is shorter than
    /// [`SIZE`](Self::SIZE).
    pub fn parse(bytes: &'a [u8]) -> Result<MessageHeader<'a>> {
        let (raw, _rest) = raw::MessageHeader::ref_from_prefix(bytes)
            .map_err(|_| Error::UnexpectedEof { offset: 0 })?;
        Ok(Self::from_raw(raw))
    }

    /// Wrap a header borrowed from the input.
    pub(crate) fn from_raw(raw: &'a raw::MessageHeader) -> Self {
        Self {
            raw: Cow::Borrowed(raw),
        }
    }

    /// Convert this header to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> MessageHeader<'static> {
        MessageHeader {
            raw: Cow::Owned(self.raw.into_owned()),
        }
    }

    /// The raw size of this segment in half-words, as it appears on the wire.
    ///
    /// This only describes this segment's size, though there could be multiple segments. A value
    /// of 65535 (0xFFFF) is a sentinel indicating a variable-length message, whose full size is
    /// determined differently; see [`message_size_bytes()`](Self::message_size_bytes) and
    /// [`segment_count()`](Self::segment_count).
    pub fn segment_size_raw(&self) -> u16 {
        self.raw.segment_size.get()
    }

    /// The raw redundant channel value, as it appears on the wire. See
    /// [`redundant_channel()`](Self::redundant_channel) for the decoded value.
    pub fn redundant_channel_raw(&self) -> u8 {
        self.raw.redundant_channel
    }

    /// The raw message type number, as it appears on the wire. See
    /// [`message_type()`](Self::message_type) for the decoded value.
    pub fn message_type_raw(&self) -> u8 {
        self.raw.message_type
    }

    /// Message sequence number.
    pub fn sequence_number(&self) -> u16 {
        self.raw.sequence_number.get()
    }

    /// If this message is [fixed-frame](Self::is_fixed_frame), this indicates this message segment's
    /// size in bytes. Otherwise, this returns [None] and
    /// [`message_size_bytes()`](Self::message_size_bytes) should be used to determine the
    /// message's full size.
    pub fn segment_size_bytes(&self) -> Option<u32> {
        let segment_size = self.raw.segment_size.get();
        if segment_size < VARIABLE_LENGTH_MESSAGE_SIZE {
            Some(segment_size as u32 * 2)
        } else {
            None
        }
    }

    /// Whether the RDA is operating on a redundant channel.
    pub fn redundant_channel(&self) -> RedundantChannel {
        match self.raw.redundant_channel {
            0 => RedundantChannel::LegacySingleChannel,
            1 => RedundantChannel::LegacyRedundantChannel1,
            2 => RedundantChannel::LegacyRedundantChannel2,
            8 => RedundantChannel::OrdaSingleChannel,
            9 => RedundantChannel::OrdaRedundantChannel1,
            10 => RedundantChannel::OrdaRedundantChannel2,
            other => RedundantChannel::Unknown(other),
        }
    }

    /// Message type discriminator.
    pub fn message_type(&self) -> MessageType {
        MessageType::from_number(self.raw.message_type)
    }

    /// This message's date and time in UTC.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(
            self.raw.date.get(),
            Duration::milliseconds(self.raw.time.get() as i64),
        )
    }

    /// Whether this message uses fixed-frame framing (2432-byte frames).
    ///
    /// Fixed-frame messages occupy one or more fixed-size frames. Variable-length messages
    /// (where this returns `false`) carry their own size and are not padded to frame
    /// boundaries. Digital Radar Data (Type 31) is always variable-length.
    pub fn is_fixed_frame(&self) -> bool {
        // ICD Table II: segment_size == 0xFFFF indicates variable-length framing.
        //
        // Type 31 (Digital Radar Data) is also variable-length, but in practice its
        // segment_size field contains the actual content halfwords rather than the
        // ICD-specified 0xFFFF sentinel. We treat Type 31 as variable-length regardless.
        self.raw.segment_size < VARIABLE_LENGTH_MESSAGE_SIZE
            && self.message_type() != MessageType::DigitalRadarData
    }

    /// If the message is [fixed-frame](Self::is_fixed_frame), this indicates the number of segments in
    /// the full message, otherwise this returns [None].
    /// [`message_size_bytes()`](Self::message_size_bytes) can be used to determine the message's
    /// full size.
    pub fn segment_count(&self) -> Option<u16> {
        if self.is_fixed_frame() {
            Some(self.raw.segment_count.get())
        } else {
            None
        }
    }

    /// If the message is [fixed-frame](Self::is_fixed_frame), this indicates this segment's
    /// number/sequence in the message, otherwise this returns [None].
    /// [`message_size_bytes()`](Self::message_size_bytes) can be used to determine the message's
    /// full size.
    pub fn segment_number(&self) -> Option<u16> {
        if self.is_fixed_frame() {
            Some(self.raw.segment_number.get())
        } else {
            None
        }
    }

    /// The full size of the message in bytes. If the message is [fixed-frame](Self::is_fixed_frame)
    /// then this is the segment size, otherwise this is the full variable-length message size.
    ///
    /// For variable-length messages (segment size = 0xFFFF), the segment count and segment number
    /// fields are repurposed as a 32-bit message size in bytes per ICD Table II.
    ///
    /// Note: this uses the raw segment size rather than [`is_fixed_frame()`](Self::is_fixed_frame)
    /// because Type 31 messages store their halfword count in the segment size even though they
    /// use variable-length framing.
    pub fn message_size_bytes(&self) -> u32 {
        let segment_size = self.raw.segment_size.get();
        if segment_size < VARIABLE_LENGTH_MESSAGE_SIZE {
            // segment_size field is in halfwords
            segment_size as u32 * 2
        } else {
            // Variable-length: segment_count and segment_number fields are repurposed
            // as a 32-bit message size in bytes (ICD Table II)
            let high = self.raw.segment_count.get() as u32;
            let low = self.raw.segment_number.get() as u32;
            (high << 16) | low
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_bytes() -> [u8; MessageHeader::SIZE] {
        let mut bytes = [0u8; MessageHeader::SIZE];
        bytes[12..14].copy_from_slice(&1216u16.to_be_bytes()); // segment size (halfwords)
        bytes[14] = 8; // redundant channel
        bytes[15] = 2; // message type
        bytes[16..18].copy_from_slice(&42u16.to_be_bytes()); // sequence number
        bytes[18..20].copy_from_slice(&20089u16.to_be_bytes()); // date
        bytes[20..24].copy_from_slice(&43_200_000u32.to_be_bytes()); // time
        bytes[24..26].copy_from_slice(&1u16.to_be_bytes()); // segment count
        bytes[26..28].copy_from_slice(&1u16.to_be_bytes()); // segment number
        bytes
    }

    #[test]
    fn size_matches_raw_layout() {
        assert_eq!(MessageHeader::SIZE, size_of::<raw::MessageHeader>());
    }

    #[test]
    fn parse_reads_fields() {
        let bytes = header_bytes();
        let header = MessageHeader::parse(&bytes).expect("parses");
        assert_eq!(header.segment_size_raw(), 1216);
        assert_eq!(header.redundant_channel_raw(), 8);
        assert_eq!(
            header.redundant_channel(),
            RedundantChannel::OrdaSingleChannel
        );
        assert_eq!(header.message_type_raw(), 2);
        assert_eq!(header.message_type(), MessageType::RdaStatusData);
        assert_eq!(header.sequence_number(), 42);
        assert!(header.is_fixed_frame());
        assert_eq!(header.segment_count(), Some(1));
        assert_eq!(header.segment_number(), Some(1));
        assert_eq!(header.message_size_bytes(), 2432);
        assert!(header.date_time().is_some());
    }

    #[test]
    fn parse_ignores_trailing_bytes() {
        let mut bytes = header_bytes().to_vec();
        bytes.extend_from_slice(&[0xAB; 16]);
        let header = MessageHeader::parse(&bytes).expect("parses");
        assert_eq!(header.sequence_number(), 42);
    }

    #[test]
    fn parse_rejects_short_input() {
        let bytes = header_bytes();
        assert!(matches!(
            MessageHeader::parse(&bytes[..MessageHeader::SIZE - 1]),
            Err(Error::UnexpectedEof { offset: 0 })
        ));
    }

    #[test]
    fn into_owned_preserves_contents() {
        let bytes = header_bytes();
        let header = MessageHeader::parse(&bytes).expect("parses");
        let owned: MessageHeader<'static> = header.clone().into_owned();
        assert_eq!(owned, header);
        assert_eq!(format!("{:?}", owned), format!("{:?}", header));
    }
}
