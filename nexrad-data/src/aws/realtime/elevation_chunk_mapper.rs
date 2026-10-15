use nexrad_decode::messages::volume_coverage_pattern;

/// Metadata describing a chunk's position within the volume scan.
///
/// Each chunk in a real-time NEXRAD volume has a sequence number (1-based).
/// Sequence 1 is always the Start chunk containing metadata (VCP, site info).
/// Subsequent sequences contain radar data, grouped by elevation sweep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ChunkMetadata {
    /// The 1-based elevation number this chunk belongs to, or None for the Start chunk.
    elevation_number: Option<usize>,
    /// Total number of chunks in this sweep (3 for standard, 6 for super-resolution).
    chunks_in_sweep: usize,
    /// Whether this is the first chunk in a sweep (inter-sweep gap applies before this chunk).
    is_first_in_sweep: bool,
    /// Azimuth rotation rate for this elevation in degrees/second (from VCP).
    azimuth_rate_dps: f64,
    /// Elevation angle in degrees (from VCP).
    elevation_angle_deg: f64,
    /// Whether this is the Start chunk (sequence 1, metadata-only).
    is_start_chunk: bool,
}

impl ChunkMetadata {
    /// The 1-based elevation number this chunk belongs to, or None for the Start chunk.
    pub(crate) fn elevation_number(&self) -> Option<usize> {
        self.elevation_number
    }

    /// Total number of chunks in this sweep (3 for standard, 6 for super-resolution).
    pub(crate) fn chunks_in_sweep(&self) -> usize {
        self.chunks_in_sweep
    }

    /// Whether this is the first chunk in a sweep.
    pub(crate) fn is_first_in_sweep(&self) -> bool {
        self.is_first_in_sweep
    }

    /// Azimuth rotation rate for this elevation in degrees/second.
    pub(crate) fn azimuth_rate_dps(&self) -> f64 {
        self.azimuth_rate_dps
    }

    /// Elevation angle in degrees.
    pub(crate) fn elevation_angle_deg(&self) -> f64 {
        self.elevation_angle_deg
    }

    /// Whether this is the Start chunk (sequence 1, metadata-only).
    pub(crate) fn is_start_chunk(&self) -> bool {
        self.is_start_chunk
    }
}

/// Maps between real-time chunk sequence numbers and volume coverage pattern elevation numbers.
#[derive(Debug)]
pub(crate) struct ElevationChunkMapper {
    // Index is elevation number - 1, value is chunk range inclusive
    elevation_chunk_mappings: Vec<(usize, usize)>,
    // Metadata for every chunk, indexed by (sequence - 1)
    chunk_metadata: Vec<ChunkMetadata>,
}

impl ElevationChunkMapper {
    /// Create a new mapper from a volume coverage pattern.
    pub(crate) fn new(vcp: &volume_coverage_pattern::Message) -> Self {
        let mut elevation_chunk_mappings = Vec::new();
        let mut chunk_metadata = Vec::new();

        // Sequence 1 is the Start chunk (metadata-only)
        chunk_metadata.push(ChunkMetadata {
            elevation_number: None,
            chunks_in_sweep: 1,
            is_first_in_sweep: false,
            azimuth_rate_dps: 0.0,
            elevation_angle_deg: 0.0,
            is_start_chunk: true,
        });

        let mut total_chunk_count = 2;
        for (elev_idx, elevation) in vcp.elevations().iter().enumerate() {
            let elevation_chunk_count = if elevation.super_resolution_half_degree_azimuth() {
                6 // 720 radials / 120 chunks per chunk
            } else {
                3 // 360 radials / 120 chunks per chunk
            };

            let start_seq = total_chunk_count;
            let end_seq = total_chunk_count + elevation_chunk_count - 1;
            elevation_chunk_mappings.push((start_seq, end_seq));

            let azimuth_rate = elevation.azimuth_rate_degrees_per_second();
            let elevation_angle = elevation.elevation_angle_degrees();

            for chunk_idx in 0..elevation_chunk_count {
                chunk_metadata.push(ChunkMetadata {
                    elevation_number: Some(elev_idx + 1),
                    chunks_in_sweep: elevation_chunk_count,
                    is_first_in_sweep: chunk_idx == 0,
                    azimuth_rate_dps: azimuth_rate,
                    elevation_angle_deg: f64::from(elevation_angle),
                    is_start_chunk: false,
                });
            }

            total_chunk_count += elevation_chunk_count;
        }

        Self {
            elevation_chunk_mappings,
            chunk_metadata,
        }
    }

    /// Get the elevation number for a given sequence number. Returns None if the sequence number
    /// does not correspond to a radar scan described by the VCP.
    pub(crate) fn get_sequence_elevation_number(&self, sequence: usize) -> Option<usize> {
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
    pub(crate) fn final_sequence(&self) -> usize {
        self.elevation_chunk_mappings
            .last()
            .map(|(_, end)| *end)
            .unwrap_or(0)
    }

    /// Get rich metadata for a specific chunk sequence number.
    ///
    /// Returns None if the sequence number is out of range.
    pub(crate) fn get_chunk_metadata(&self, sequence: usize) -> Option<&ChunkMetadata> {
        if sequence == 0 || sequence > self.chunk_metadata.len() {
            return None;
        }
        Some(&self.chunk_metadata[sequence - 1])
    }
}
