//!
//! Message type 2 "RDA Status Data" contains information about the current RDA state, system
//! control, operating status, scanning strategy, performance parameters like transmitter power and
//! calibration, and system alarms. This message is sent upon wideband connection, after state or
//! control changes, at the beginning of each volume scan, and after an RPG request.
//!

pub mod alarm;

mod message;
pub use message::Message;

mod raw;
pub use raw::*;
