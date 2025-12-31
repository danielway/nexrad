/// Indicates whether the message is compressed and what type of compression was used.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CompressionIndicator {
    Uncompressed,
    CompressedBZIP2,
    CompressedZLIB,
    FutureUse,
}
