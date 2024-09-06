//!
//! Contains the Result and Error types for NEXRAD model operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("two sweeps' elevation numbers do not match")]
    ElevationMismatchError,
}
