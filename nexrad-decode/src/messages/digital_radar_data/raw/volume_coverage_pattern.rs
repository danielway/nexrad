/// Volume coverage pattern (VCP) definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolumeCoveragePattern {
    VCP12,
    VCP31,
    VCP35,
    VCP112,
    VCP212,
    VCP215,
}
