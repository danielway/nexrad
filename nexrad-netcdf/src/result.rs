//! Error types and Results for NetCDF operations.

use thiserror::Error;

/// Errors that can occur during NetCDF operations.
#[derive(Debug, Error)]
pub enum Error {}

/// A specialized Result type for NetCDF operations.
pub type Result<T> = std::result::Result<T, Error>;
