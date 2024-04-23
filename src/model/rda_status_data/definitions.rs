/// Acknowledgement of command receipt by RDA system.
pub enum CommandAcknowledgement {
    RemoteVCPReceived,
    ClutterBypassMapReceived,
    ClutterCensorZonesReceived,
    RedundantChannelControlCommandAccepted,
}

/// The possible RDA system clutter mitigation decision statuses.
pub enum ClutterMitigationDecisionStatus {
    Disabled,
    Enabled,
    /// Which elevation segments of the bypass map are applied.
    BypassMapElevationSegments(Vec<u8>),
}

/// The possible RDA system auxiliary power generator states.
pub enum AuxiliaryPowerGeneratorState {
    SwitchedToAuxiliaryPower,
    UtilityPowerAvailable,
    GeneratorOn,
    TransferSwitchSetToManual,
    CommandedSwitchover,
}

/// The possible RDA system control authorizations.
pub enum ControlAuthorization {
    NoAction,
    LocalControlRequested,
    RemoteControlRequested,
}

/// The possible RDA system control statuses.
pub enum ControlStatus {
    LocalControlOnly,
    RemoteControlOnly,
    EitherLocalOrRemoteControl,
}

/// The possible RDA system operability statuses.
pub enum OperabilityStatus {
    OnLine,
    MaintenanceActionRequired,
    MaintenanceActionMandatory,
    CommandedShutDown,
    Inoperable,
}

/// The possible RDA system operational modes.
pub enum OperationalMode {
    Operational,
    Maintenance,
}

/// The RDA system's performance check status.
pub enum PerformanceCheckStatus {
    NoCommandPending,
    ForcePerformanceCheckPending,
    InProgress,
}

/// The RDA system's RMS control status.
pub enum RMSControlStatus {
    NonRMS,
    RMSInControl,
    RDAInControl,
}

/// Indicates whether this is the RDA system's controlling channel.
pub enum SpotBlankingStatus {
    NotInstalled,
    Enabled,
    Disabled,
}

/// The possible RDA system statuses.
pub enum RDAStatus {
    StartUp,
    Standby,
    Restart,
    Operate,
    Spare,
}

/// Whether the RDA system has super resolution enabled.
pub enum SuperResolutionStatus {
    Enabled,
    Disabled,
}

/// The possible RDA system transition power source statuses.
pub enum TransitionPowerSourceStatus {
    NotInstalled,
    Off,
    OK,
    Unknown,
}
