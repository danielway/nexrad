/// The possible RDA redundant channels.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RedundantChannel {
    LegacySingleChannel,
    LegacyRedundantChannel1,
    LegacyRedundantChannel2,
    ORDASingleChannel,
    ORDARedundantChannel1,
    ORDARedundantChannel2,
}
