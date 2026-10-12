/// Processing status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessingStatus {
    /// Receiver noise (RxR Noise) processing.
    RxrNoise,
    /// Clutter Bypass Thresholding (CBT) processing.
    Cbt,
    /// Other processing status value.
    Other(u16),
}
