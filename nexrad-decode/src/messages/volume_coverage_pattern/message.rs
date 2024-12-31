use crate::messages::volume_coverage_pattern::{ElevationDataBlock, Header};

/// The digital radar data message includes base radar data from a single radial for various
/// products.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// The decoded volume coverage pattern header.
    pub header: Header,

    /// The decoded elevation data blocks.
    pub elevations: Vec<ElevationDataBlock>,
}

impl Message {
    /// Create a new volume coverage pattern message
    pub(crate) fn new(header: Header, elevations: Vec<ElevationDataBlock>) -> Self {
        Self { header, elevations }
    }
}
