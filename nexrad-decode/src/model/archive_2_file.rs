use crate::model::messages::MessageWithHeader;
use crate::model::Archive2Header;

/// An Archive II file containing decoded NEXRAD Level II data.
#[derive(Debug)]
pub struct Archive2File {
    /// The volume scan header.
    pub header: Archive2Header,

    // todo: are these actually records? are there record headers or just message headers?
    //       the archive II docs mention a "metadata record" with multiple messages; are we handling
    //       that correctly?
    /// The decoded messages.
    pub messages: Vec<MessageWithHeader>,
}
