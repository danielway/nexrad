//! 
//! Message type 2 "RDA Status Data" includes information about the current RDA state, system
//! control, operating status, scanning strategy, performance parameters, calibration, and alarms.
//! This message is included in a variety of scenarios including at the beginning of each volume
//! scan.
//! 

pub mod alarm;

mod data_transmission_enabled;
pub use data_transmission_enabled::DataTransmissionEnabled;

mod scan_data_flags;
pub use scan_data_flags::ScanDataFlags;

mod definitions;
pub use definitions::*;

mod rda_status_message;
pub use rda_status_message::RDAStatusData;

mod volume_coverage_pattern;

pub use volume_coverage_pattern::VolumeCoveragePatternNumber;
