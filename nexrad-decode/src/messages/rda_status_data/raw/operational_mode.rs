/// The possible RDA system operational modes.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum OperationalMode {
    Operational,
    Maintenance,
}
