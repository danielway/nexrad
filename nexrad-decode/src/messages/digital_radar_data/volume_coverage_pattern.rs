/// Volume coverage pattern (VCP) definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolumeCoveragePattern {
    /// VCP 12 - Precipitation mode with 14 elevations.
    Vcp12,
    /// VCP 31 - Clear air mode with long dwell times.
    Vcp31,
    /// VCP 35 - Clear air mode variant.
    Vcp35,
    /// VCP 112 - Precipitation mode, enhanced low-level coverage.
    Vcp112,
    /// VCP 212 - Precipitation mode with AVSET.
    Vcp212,
    /// VCP 215 - Precipitation mode, similar to VCP 212.
    Vcp215,
    /// Unknown VCP number for forward compatibility with new scan strategies.
    Unknown(u16),
}
