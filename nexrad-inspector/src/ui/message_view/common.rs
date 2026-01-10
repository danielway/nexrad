//! Common message header parsing for unsupported message types.

use nexrad_decode::messages::MessageHeader;

/// Parses and displays just the common message header for unsupported message types.
pub fn parse_common_header_only(header: &MessageHeader) -> String {
    let msg_type = header.message_type();
    let datetime = header
        .date_time()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let channel = header.rda_redundant_channel();

    format!(
        "=== Common Message Header ===\n\n\
         Message Type: {} ({:?})\n\
         Sequence Number: {}\n\
         Date/Time: {}\n\
         Redundant Channel: {:?}\n\
         Segment Size: {} half-words\n\
         Segmented: {}\n\
         Message Size: {} bytes\n\n\
         (No additional parsing available for this message type)",
        header.message_type,
        msg_type,
        header.sequence_number,
        datetime,
        channel,
        header.segment_size,
        header.segmented(),
        header.message_size_bytes()
    )
}
