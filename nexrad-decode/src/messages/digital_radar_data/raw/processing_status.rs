/// Processing status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessingStatus {
    RxRNoise,
    CBT,
    Other(u16),
}
