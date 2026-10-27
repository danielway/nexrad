/// Azimuthal spacing between adjacent radials.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum AzimuthResolutionSpacing {
    /// 0.5 degree spacing between radials.
    HalfDegree,
    /// 1.0 degree spacing between radials.
    OneDegree,
    /// Unknown spacing value for forward compatibility.
    Unknown(u8),
}
