/// The possible RDA system control authorizations.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlAuthorization {
    NoAction,
    LocalControlRequested,
    RemoteControlRequested,
}
