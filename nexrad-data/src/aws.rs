//!
//! Downloads archival and real-time NEXRAD level II weather radar data from AWS Open Data buckets
//! populated by NOAA.
//!
//! [AWS Open Data NOAA NEXRAD](https://registry.opendata.aws/noaa-nexrad/)
//!
//! [AWS Labs Open Data Documentation](https://github.com/awslabs/open-data-docs/tree/main/docs/noaa/noaa-nexrad)
//!
//! **NEXRAD Level II archive data**: `arn:aws:s3:::unidata-nexrad-level2`
//!
//! **NEXRAD Level II real-time data**: `arn:aws:s3:::unidata-nexrad-level2-chunks`
//!

pub mod archive;
pub mod realtime;

mod s3;
