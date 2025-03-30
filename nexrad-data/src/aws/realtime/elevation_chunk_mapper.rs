use crate::aws::realtime::ChunkType;
use nexrad_decode::messages::volume_coverage_pattern;

/// Maps between real-time chunk sequence numbers and volume coverage pattern elevation numbers.
pub struct ElevationChunkMapper {
    vcp: volume_coverage_pattern::Message,

    // Index is elevation number - 1, value is chunk range inclusive
    elevation_chunk_mappings: Vec<(usize, usize)>,
}

impl ElevationChunkMapper {
    /// Create a new mapper from a volume coverage pattern.
    pub fn new(vcp: volume_coverage_pattern::Message) -> Self {
        let mut elevation_chunk_mappings = Vec::new();
        let mut total_chunk_count = 0;
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
            vcp,
            elevation_chunk_mappings,
        }
    }

    /// Get this mapper's volume coverage pattern.
    pub fn get_vcp(&self) -> &volume_coverage_pattern::Message {
        &self.vcp
    }

    /// Get the elevation number for a given sequence number. Returns None if the sequence number   
    /// does not correspond to a radar scan described by the VCP.
    pub fn get_sequence_elevation(
        &self,
        sequence: usize,
    ) -> Option<&volume_coverage_pattern::ElevationDataBlock> {
        // The first chunk is metadata, not a radar scan described by the VCP
        if sequence == 1 {
            return None;
        }

        self.elevation_chunk_mappings
            .iter()
            .position(|(start, end)| sequence > *start && sequence <= *end)
            .map(|index| &self.vcp.elevations[index])
    }

    /// Get the type of chunk for a given sequence number.
    pub fn get_sequence_type(&self, sequence: usize) -> ChunkType {
        if sequence == 1 {
            ChunkType::Start
        } else if sequence
            == self
                .elevation_chunk_mappings
                .last()
                .map_or(0, |(_, end)| *end)
        {
            ChunkType::End
        } else {
            ChunkType::Intermediate
        }
    }

    /// Get the sequence number range for a given elevation number. Returns None if the elevation
    /// number does not correspond to a radar scan described by the VCP.
    pub fn get_elevation_sequence_range(&self, elevation: usize) -> Option<(usize, usize)> {
        self.elevation_chunk_mappings.get(elevation).cloned()
    }
}
