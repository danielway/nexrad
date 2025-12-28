//!
//! Message type 2 "RDA Status Data" contains information about the current RDA state, system
//! control, operating status, scanning strategy, performance parameters like transmitter power and
//! calibration, and system alarms. This message is sent upon wideband connection, after state or
//! control changes, at the beginning of each volume scan, and after an RPG request.
//!

pub mod alarm;

mod data_transmission_enabled;

pub use data_transmission_enabled::DataTransmissionEnabled;
use std::io::Read;

mod scan_data_flags;
pub use scan_data_flags::ScanDataFlags;

mod raw;
pub use raw::*;

mod message;
pub use message::Message;

mod volume_coverage_pattern;
use crate::result::Result;
use crate::util::deserialize;
pub use volume_coverage_pattern::VolumeCoveragePatternNumber;

/// Decodes an RDA status message type 2 from the provided reader.
pub fn decode_rda_status_message<R: Read>(reader: &mut R) -> Result<Message> {
    deserialize(reader)
}
