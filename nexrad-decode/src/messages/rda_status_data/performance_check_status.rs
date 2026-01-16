/// The RDA system's performance check status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PerformanceCheckStatus {
    NoCommandPending,
    ForcePerformanceCheckPending,
    InProgress,
    /// Unknown performance check status value for forward compatibility.
    Unknown(u16),
}
