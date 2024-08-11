//!
//! Message type 2 "RDA Status Data" contains information about the current RDA state, system
//! control, operating status, scanning strategy, performance parameters like transmitter power and
//! calibration, and system alarms. This message is sent upon wideband connection, after state or
//! control changes, at the beginning of each volume scan, and after an RPG request.
//!

pub mod alarm;

mod data_transmission_enabled;
pub use data_transmission_enabled::DataTransmissionEnabled;

mod scan_data_flags;
pub use scan_data_flags::ScanDataFlags;

mod definitions;
pub use definitions::*;

mod message;
pub use message::Message;

mod volume_coverage_pattern;

pub use volume_coverage_pattern::VolumeCoveragePatternNumber;
