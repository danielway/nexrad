/// The possible RDA system control statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlStatus {
    LocalControlOnly,
    RemoteControlOnly,
    EitherLocalOrRemoteControl,
}
