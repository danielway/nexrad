//!
//! Struct definitions for chunk meta, encoded binary data, and decoded data structures.
//!

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// An encoded (and possibly-compressed) NEXRAD WSR-88D chunk file including sweep data. If
/// compressed, the data is compressed using BZIP2. See
/// [decompress_chunk](crate::decompress::decompress_chunk) and
/// [decode_chunk](crate::decode::decode_chunk).
#[derive(Serialize, Deserialize)]
pub struct EncodedChunk {
    data: Vec<u8>,
}

impl EncodedChunk {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// The raw, encoded, and possibly-compressed data for this chunk.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

/// A decoded NEXRAD WSR-88D chunk file including sweep data.
#[derive(Serialize, Deserialize, Debug)]
pub struct Chunk {
    file_header: FileHeader,
}

impl Chunk {
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