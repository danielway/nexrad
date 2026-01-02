/// The RDA system's RMS control status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RMSControlStatus {
    NonRMS,
    RMSInControl,
    RDAInControl,
}
