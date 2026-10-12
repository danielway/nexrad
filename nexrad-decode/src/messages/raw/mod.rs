mod redundant_channel;
pub use redundant_channel::RedundantChannel;

mod message_header;
pub(crate) use message_header::{MessageHeader, VARIABLE_LENGTH_MESSAGE_SIZE};

pub(crate) mod primitive_aliases;
