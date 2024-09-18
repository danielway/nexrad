/// Acknowledgement of command receipt by RDA system.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CommandAcknowledgement {
    RemoteVCPReceived,
    ClutterBypassMapReceived,
    ClutterCensorZonesReceived,
    RedundantChannelControlCommandAccepted,
}

/// The possible RDA system clutter mitigation decision statuses.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ClutterMitigationDecisionStatus {
    Disabled,
    Enabled,
    /// Which elevation segments of the bypass map are applied.
    BypassMapElevationSegments(Vec<u8>),
}

/// The possible RDA system auxiliary power generator states.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AuxiliaryPowerGeneratorState {
    SwitchedToAuxiliaryPower,
    UtilityPowerAvailable,
    GeneratorOn,
    TransferSwitchSetToManual,
    CommandedSwitchover,
}

/// The possible RDA system control authorizations.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlAuthorization {
    NoAction,
    LocalControlRequested,
    RemoteControlRequested,
}

/// The possible RDA system control statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlStatus {
    LocalControlOnly,
    RemoteControlOnly,
    EitherLocalOrRemoteControl,
}

/// The possible RDA system operability statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum OperabilityStatus {
    OnLine,
    MaintenanceActionRequired,
    MaintenanceActionMandatory,
    CommandedShutDown,
    Inoperable,
}

/// The possible RDA system operational modes.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum OperationalMode {
    Operational,
    Maintenance,
}

/// The RDA system's performance check status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PerformanceCheckStatus {
    NoCommandPending,
    ForcePerformanceCheckPending,
    InProgress,
}

/// The RDA system's RMS control status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RMSControlStatus {
    NonRMS,
    RMSInControl,
    RDAInControl,
}

/// Indicates whether this is the RDA system's controlling channel.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SpotBlankingStatus {
    NotInstalled,
    Enabled,
    Disabled,
}

/// The possible RDA system statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RDAStatus {
    StartUp,
    Standby,
    Restart,
    Operate,
    Spare,
}

/// Whether the RDA system has super resolution enabled.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SuperResolutionStatus {
    Enabled,
    Disabled,
}

/// The possible RDA system transition power source statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TransitionPowerSourceStatus {
    NotInstalled,
    Off,
    OK,
    Unknown,
}
