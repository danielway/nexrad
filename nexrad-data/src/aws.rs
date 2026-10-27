//! AWS data access for NEXRAD weather radar files.
//!
//! This module provides functions to download NEXRAD data from AWS Open Data buckets
//! populated by NOAA. Both historical archive data and real-time streaming data are available.
//!
//! # Data Sources
//!
//! - `archive` - Historical volume files from the NEXRAD Level II archive
//! - `realtime` - Live radar data with chunk-based streaming
//!
//! # AWS Buckets
//!
//! | Bucket | ARN | Description |
//! |--------|-----|-------------|
//! | Archive | `arn:aws:s3:::unidata-nexrad-level2` | Historical volumes |
//! | Real-time | `arn:aws:s3:::unidata-nexrad-level2-chunks` | Live data chunks |
//!
//! # HTTP client and timeouts
//!
//! Every request goes through one shared HTTP client. On native targets it applies these
//! timeouts:
//!
//! | Timeout | Value | Applies to |
//! |---------|-------|------------|
//! | Connect | 30 s | Establishing a connection |
//! | Read | 60 s | Waiting for the next bytes of a response |
//! | Request | 300 s | A whole request, from sending it to reading the last byte of the body |
//!
//! A request that exceeds one fails with a transport error, which
//! [`AwsError::is_transient`](crate::result::AwsError::is_transient) reports as transient. The
//! values are not configurable yet. On wasm32 the client uses the browser's `fetch`, these
//! timeouts do not apply, and the browser's own limits do.
//!
//! # Resources
//!
//! - [AWS Open Data NOAA NEXRAD](https://registry.opendata.aws/noaa-nexrad/)
//! - [AWS Labs Documentation](https://github.com/awslabs/open-data-docs/tree/main/docs/noaa/noaa-nexrad)

pub mod archive;
pub mod realtime;

mod client;
mod s3;
