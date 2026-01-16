/// The possible RDA redundant channels.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RedundantChannel {
    LegacySingleChannel,
    LegacyRedundantChannel1,
    LegacyRedundantChannel2,
    ORDASingleChannel,
    ORDARedundantChannel1,
    ORDARedundantChannel2,
    /// Unknown redundant channel value for forward compatibility.
    Unknown(u8),
}
