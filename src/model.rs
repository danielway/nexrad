//!
//! Struct definitions for decoded NEXRAD Level II data structures as defined by NOAA's WSR-88D
//! Interface Control Document for Archive II.
//!
//! These API definitions should match the ICD 2620010H as of build 19.0.
//!

mod archive2_header;
pub use archive2_header::Archive2Header;

mod definitions;
mod message_header;
mod message_type;
mod primitive_aliases;
mod util;

pub mod messages;
