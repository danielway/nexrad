use crate::messages::volume_coverage_pattern::{ElevationDataBlock, Header};
use crate::result::Result;
use crate::slice_reader::SliceReader;
use std::borrow::Cow;

/// The volume coverage pattern message describes the current scan pattern. This is sent on
/// wideband connection and the start of each volume scan.
///
/// This message's contents correspond to ICD 2620002AA section 3.2.4.12 Table XI.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    /// The decoded volume coverage pattern header.
    pub header: Cow<'a, Header>,

    /// The decoded elevation data blocks.
    pub elevations: Cow<'a, [ElevationDataBlock]>,
}

impl<'a> Message<'a> {
    /// Parse a volume coverage pattern message from the input.
    pub(crate) fn parse(reader: &mut SliceReader<'a>) -> Result<Self> {
        let header = reader.take_ref::<Header>()?;

        let elevation_cuts = header.number_of_elevation_cuts.get() as usize;
        let elevations = reader.take_slice::<ElevationDataBlock>(elevation_cuts)?;

        Ok(Self {
            header: Cow::Borrowed(header),
            elevations: Cow::Borrowed(elevations),
        })
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: Cow::Owned(self.header.into_owned()),
            elevations: Cow::Owned(self.elevations.into_owned()),
        }
    }
}
