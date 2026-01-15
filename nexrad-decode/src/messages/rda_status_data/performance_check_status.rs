/// The RDA system's performance check status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PerformanceCheckStatus {
    NoCommandPending,
    ForcePerformanceCheckPending,
    InProgress,
}
