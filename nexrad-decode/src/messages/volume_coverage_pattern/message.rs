use crate::messages::volume_coverage_pattern::{ElevationDataBlock, Header};
use crate::result::Result;
use crate::util::{take_ref, take_slice};
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
    pub(crate) fn parse<'b>(input: &'b mut &'a [u8]) -> Result<Self> {
        let header = take_ref::<Header>(input)?;
        let elevations = take_slice::<ElevationDataBlock>(
            input,
            header.number_of_elevation_cuts.get() as usize,
        )?;

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
