use crate::messages::{MessageContents, MessageHeader};

/// A decoded NEXRAD Level II message with its metadata header.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    header: &'a MessageHeader,
    contents: MessageContents<'a>,
}

impl<'a> Message<'a> {
    /// Create a new unsegmented message.
    pub(crate) fn unsegmented(header: MessageHeader, contents: MessageContents) -> Self {
        Self { header, contents }
    }

    /// This message's header.
    pub fn header(&self) -> &MessageHeader {
        &self.header
    }

    /// This message's contents.
    pub fn contents(&self) -> &MessageContents {
        &self.contents
    }

    /// Consume this message, returning ownership of its contents.
    pub fn into_contents(self) -> MessageContents<'a> {
        self.contents
    }
}
