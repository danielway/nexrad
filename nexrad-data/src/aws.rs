//!
//! Downloads NEXRAD level-II data from an AWS S3 bucket populated by NOAA.
//!

pub mod archive;
pub mod realtime;

mod s3;
mod search;
