/// The possible RDA system statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RDAStatus {
    StartUp,
    Standby,
    Restart,
    Operate,
    Spare,
}
