use crate::messages::volume_coverage_pattern::{ElevationDataBlock, Header};
use crate::result::Result;
use crate::util::take_ref;

/// The volume coverage pattern message describes the current scan pattern. This is sent on
/// wideband connection and the start of each volume scan.
///
/// This message's contents correspond to ICD 2620002AA section 3.2.4.12 Table XI.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    /// The decoded volume coverage pattern header.
    pub header: &'a Header,

    /// The decoded elevation data blocks.
    pub elevations: Vec<&'a ElevationDataBlock>,
}

impl<'a> Message<'a> {
    /// Parse a volume coverage pattern message from the input.
    pub(crate) fn parse<'b>(input: &'b mut &'a [u8]) -> Result<Self> {
        let header = take_ref::<Header>(input)?;

        let mut elevations: Vec<&ElevationDataBlock> = Vec::new();
        for _ in 0..header.number_of_elevation_cuts.get() {
            let elevation_block = take_ref::<ElevationDataBlock>(input)?;
            elevations.push(elevation_block);
        }

        Ok(Self { header, elevations })
    }
}
