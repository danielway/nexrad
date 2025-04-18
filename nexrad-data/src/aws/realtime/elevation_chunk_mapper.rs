use nexrad_decode::messages::volume_coverage_pattern;

/// Maps between real-time chunk sequence numbers and volume coverage pattern elevation numbers.
pub struct ElevationChunkMapper {
    // Index is elevation number - 1, value is chunk range inclusive
    elevation_chunk_mappings: Vec<(usize, usize)>,
}

impl ElevationChunkMapper {
    /// Create a new mapper from a volume coverage pattern.
    pub fn new(vcp: &volume_coverage_pattern::Message) -> Self {
        let mut elevation_chunk_mappings = Vec::new();
        let mut total_chunk_count = 2;
        for elevation in vcp.elevations.iter() {
            let elevation_chunk_count = if elevation.super_resolution_control_half_degree_azimuth()
            {
                6 // 720 radials / 120 chunks per chunk
            } else {
                3 // 360 radials / 120 chunks per chunk
            };

            elevation_chunk_mappings.push((
                total_chunk_count,
                total_chunk_count + elevation_chunk_count - 1,
            ));

            total_chunk_count += elevation_chunk_count;
        }

        Self {
            elevation_chunk_mappings,
        }
    }

    /// Get the elevation number for a given sequence number. Returns None if the sequence number   
    /// does not correspond to a radar scan described by the VCP.
    pub fn get_sequence_elevation_number(&self, sequence: usize) -> Option<usize> {
        // The first chunk is metadata, not a radar scan described by the VCP
        if sequence == 1 {
            return None;
        }

        self.elevation_chunk_mappings
            .iter()
            .position(|(start, end)| sequence >= *start && sequence <= *end)
            .map(|elevation_index| elevation_index + 1)
    }

    /// Returns the final sequence number for the volume.
    pub fn final_sequence(&self) -> usize {
        self.elevation_chunk_mappings
            .last()
            .map(|(_, end)| *end)
            .unwrap_or(0)
    }
}
