/// Processing status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessingStatus {
    /// Receiver noise (RxR Noise) processing.
    RxrNoise,
    /// Clutter Bypass Thresholding (CBT) processing.
    Cbt,
    /// A processing status value the ICD does not define, carrying the raw value.
    Unknown(u16),
}
