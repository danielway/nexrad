//!
//! Struct definitions for decoded NEXRAD Level II data structures.
//!

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// A decoded NEXRAD WSR-88D data file including sweep data.
#[derive(Serialize, Deserialize, Debug)]
pub struct DataFile {
    file_header: FileHeader,
}

impl DataFile {
    pub(crate) fn new(file_header: FileHeader) -> Self {
        Self { file_header }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct FileHeader {
    /// Filename of the archive.
    pub filename: [u8; 9],

    /// File extension.
    pub ext: [u8; 3],

    /// Modified Julian date of the file.
    pub file_date: u32,

    /// Milliseconds of day since midnight of the file.
    pub file_time: u32,

    /// Unused field.
    pub unused1: [u8; 4],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageHeader {
    /// 12 bytes inserted by RPG Communications Mgr. Ignored.
    pub rpg: [u8; 12],

    /// Message size for this segment, in halfwords
    pub msg_size: u16,

    /// RDA Redundant Channel
    pub channel: u8,

    /// Message type. For example, 31
    pub msg_type: u8,

    /// Msg seq num = 0 to 7FFF, then roll over to 0
    pub id_seq: u16,

    /// Modified Julian date from 1/1/70
    pub msg_date: u16,

    /// Packet generation time in ms past midnight
    pub msg_time: u32,

    /// Number of segments for this message
    pub num_segs: u16,

    /// Number of this segment
    pub seg_num: u16,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct DataHeader {
    /// Radar iste identifier
    pub radar_id: [u8; 4],

    /// Data collection time in milliseconds past midnight GMT
    pub ray_time: u32,

    /// Julian date - 2440586.5 (1/01/1970)
    pub ray_date: u16,

    /// Radial number within elevation scan
    pub azm_num: u16,

    /// Azimuth angle in degrees (0 to 359.956055)
    pub azm: f32,

    /// 0 = uncompressed, 1 = BZIP2, 2 = zlib
    pub compression_code: u8,

    /// For word alignment
    pub spare: u8,

    /// Radial length in bytes, including data header block
    pub radial_len: u16,

    /// Azimuthal resolution
    pub azm_res: u8,

    /// Radial status
    pub radial_status: u8,

    /// Elevation number
    pub elev_num: u8,

    /// Sector cut number
    pub sector_cut_num: u8,

    /// Elevation angle in degrees (-7.0 to 70.0)
    pub elev: f32,

    /// Radial spot blanking
    pub radial_spot_blanking: u8,

    /// Azimuth indexing mode
    pub azm_indexing_mode: u8,

    /// Data block count
    pub data_block_count: u16,

    /// Data block pointers
    pub vol_const: u32,
    pub elev_const: u32,
    pub radial_const: u32,
    pub field1: u32,
    pub field2: u32,
    pub field3: u32,
    pub field4: u32,
    pub field5: u32,
    pub field6: u32,
}
