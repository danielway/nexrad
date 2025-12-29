use crate::messages::clutter_filter_map::elevation_segment::ElevationSegment;
use crate::messages::clutter_filter_map::raw::Header;
use crate::result::Result;
use crate::util::take_ref;
use std::borrow::Cow;
use std::fmt::Debug;

/// A clutter filter map describing elevations, azimuths, and ranges containing clutter to
/// filtered from radar products. The RDA transmits this any time the map changes.
///
/// This message's contents correspond to ICD 2620002AA section 3.2.4.15 Table XIV.
/// The message starts with a brief header followed by a loop of elevation, azimuth,
/// and finally range/gate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message<'a> {
    /// Decoded header information for this clutter filter map.
    pub header: Cow<'a, Header>,

    /// The elevation segments defined in this clutter filter map.
    pub elevation_segments: Vec<ElevationSegment<'a>>,
}

impl<'a> Message<'a> {
    /// Parse a clutter filter map message from the input.
    pub(crate) fn parse<'b>(input: &'b mut &'a [u8]) -> Result<Self> {
        let header = take_ref::<Header>(input)?;

        let segment_count = header.elevation_segment_count.get() as u8;
        let mut message = Message {
            header: Cow::Borrowed(header),
            elevation_segments: Vec::with_capacity(segment_count as usize),
        };

        for segment_number in 0..segment_count {
            let segment = ElevationSegment::parse(input, segment_number)?;
            message.elevation_segments.push(segment);
        }

        Ok(message)
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: Cow::Owned(self.header.into_owned()),
            elevation_segments: self
                .elevation_segments
                .into_iter()
                .map(|s| s.into_owned())
                .collect(),
        }
    }
}
