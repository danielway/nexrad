use crate::result::Result;
use std::io::Read;

struct Archive2Header {
    // 9 bytes for tape filename
    // ‘AR2V0 0xx.’*
    // note: * xx indicates version where:
    // Version 02: Super Resolution disabled at the RDA (pre RDA Build 12.0)
    // Version 03: Super Resolution (pre RDA Build 12.0)
    // Version 04: Recombined Super Resolution
    // Version 05: Super Resolution disabled at the RDA (RDA Build 12.0 and later)
    // Version 06: Super Resolution (RDA Build 12.0 and later)
    // Version 07: Recombined Super Resolution (RDA Build 12.0 and later)
    // NOTE: Dual-pol data introduced in RDA Build 12.0
    tape_filename: [u8; 9],

    // 3 bytes for extension number
    extension_number: [u8; 3],

    // 4 bytes for nexrad-modified julian date
    // **Days since 1/1/1970 where 1/1/1970 equals day 1
    nexrad_modified_julian_date: [u8; 4],

    // 4 bytes for time milliseconds past midnight
    time_milliseconds_past_midnight: [u8; 4],

    // 4 bytes for icao of radar
    icao_of_radar: [u8; 4],
}

// reads the archive II header from the given reader. header is at start of volume.
fn decode_archive2_header(reader: &impl Read) -> Result<Archive2Header> {
    todo!()
}
