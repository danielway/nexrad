//!
//! Message type 5 "Volume Coverage Pattern" provides details about the volume
//! coverage pattern being used. The RDA sends the Volume Coverage Pattern message
//! upon wideband connection and at the beginning of each volume scan. The volume
//! coverage pattern message includes a header which describes how the volume is being
//! collected as well as a block for each elevation cut detailing the radar settings
//! being used for that cut.
//!

mod raw;
pub use raw::*;

mod message;
pub use message::Message;

#[cfg(test)]
mod snapshot_test;
