use crate::messages::clutter_filter_map::{Header, Segment};

/// A clutter filter map message describing elevations, azimuths, and ranges containing clutter to
/// filtered from radar products.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message {
    /// Decoded header information for this clutter filter map.
    pub header: Header,

    /// The segments composing this message.
    pub segments: Vec<Segment>,
}

impl Message {
    /// Creates a new clutter filter map message from the coded header.
    pub(crate) fn new(header: Header) -> Self {
        Self {
            header,
            segments: Vec::new(),
        }
    }
}
