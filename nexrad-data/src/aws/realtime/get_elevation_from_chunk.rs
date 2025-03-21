/// Get the elevation from a chunk based on the sequence number. Returns None if the chunk does not
/// correspond to a radar scan described by the VCP.
pub fn get_elevation_from_chunk(
    sequence: usize,
    elevations: &Vec<nexrad_decode::messages::volume_coverage_pattern::ElevationDataBlock>,
) -> Option<&nexrad_decode::messages::volume_coverage_pattern::ElevationDataBlock> {
    // The first chunk is metadata, not a radar scan described by the VCP
    if sequence == 1 {
        return None;
    }

    let mut chunk_count = 1;
    for elevation in elevations {
        let elevation_chunk_count = if elevation.super_resolution_control_half_degree_azimuth() {
            6 // 720 radials / 120 chunks per chunk
        } else {
            3 // 360 radials / 120 chunks per chunk
        };

        chunk_count += elevation_chunk_count;

        if sequence <= chunk_count {
            return Some(elevation);
        }
    }

    None
}
