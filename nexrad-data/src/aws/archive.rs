//!
//! TODO: explain how the volume bucket is structured/works
//!

mod identifier;
pub use identifier::*;

mod download_file;
pub use download_file::*;

mod list_files;
pub use list_files::*;

const ARCHIVE_BUCKET: &str = "noaa-nexrad-level2";
