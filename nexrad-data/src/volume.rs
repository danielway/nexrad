//!
//! # Archive II Volumes
//!
//! Archival NEXRAD weather radar data is distributed using an archive format built atop Unidata's
//! ["Local Data Manager" (or LDM)](https://www.unidata.ucar.edu/software/ldm/) system. Archive
//! files called "volumes" contain NEXRAD Level II radar data and are composed of LDM records. They
//! start with a "volume header record" that provides basic metadata about the radar site and
//! collection time followed by a series of compressed records that contain radar messages with
//! data.
//!
//! The document "Interface Control Document for the Archive II/User" 2620010H (build 19.0 at
//! writing) describes this archive format in detail, particularly in section 7 "Archive II
//! Application Layer".
//!

mod identifier;
pub use identifier::*;

mod file;
pub use file::*;

mod header;
pub use header::*;

mod record;
pub use record::*;

mod util;
