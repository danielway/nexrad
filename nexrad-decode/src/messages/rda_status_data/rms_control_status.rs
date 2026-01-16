/// The RDA system's RMS control status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RMSControlStatus {
    NonRMS,
    RMSInControl,
    RDAInControl,
    /// Unknown RMS control status value for forward compatibility.
    Unknown(u16),
}
