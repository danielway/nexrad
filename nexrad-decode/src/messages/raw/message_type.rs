/// The types of data messages transferred between the RDA and RPG.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
#[repr(u8)]
#[non_exhaustive]
pub enum MessageType {
    /// Replaced by message type 31.
    RdaDigitalRadarData = 1,

    /// Metadata.
    RdaStatusData = 2,

    /// Metadata.
    RdaPerformanceMaintenanceData = 3,

    /// Console message from RDA.
    RdaConsoleMessage = 4,

    /// Metadata.
    RdaVolumeCoveragePattern = 5,

    /// Control commands to RDA.
    RdaControlCommands = 6,

    /// VCP from RPG.
    RpgVolumeCoveragePattern = 7,

    /// Clutter censor zones from RPG.
    RpgClutterCensorZones = 8,

    /// Data request from RPG.
    RpgRequestForData = 9,

    /// Console message from RPG.
    RpgConsoleMessage = 10,

    /// Loopback test from RDA.
    RdaLoopBackTest = 11,

    /// Loopback test from RPG.
    RpgLoopBackTest = 12,

    /// No longer sent.
    RdaClutterFilterBypassMap = 13,

    /// Spare message type.
    Spare1 = 14,

    /// Metadata.
    RdaClutterFilterMap = 15,

    /// Reserved for FAA RMS use.
    ReservedFaaRmsOnly1 = 16,

    /// Reserved for FAA RMS use.
    ReservedFaaRmsOnly2 = 17,

    /// Metadata.
    RdaAdaptationData = 18,

    /// Reserved message type.
    Reserved1 = 20,

    /// Reserved message type.
    Reserved2 = 21,

    /// Reserved message type.
    Reserved3 = 22,

    /// Reserved message type.
    Reserved4 = 23,

    /// Reserved for FAA RMS use.
    ReservedFaaRmsOnly3 = 24,

    /// Reserved for FAA RMS use.
    ReservedFaaRmsOnly4 = 25,

    /// Reserved for FAA RMS use.
    ReservedFaaRmsOnly5 = 26,

    /// Reserved message type.
    Reserved5 = 29,

    /// Generic format digital radar data (primary data format).
    RdaDigitalRadarDataGenericFormat = 31,

    /// PRF data from RDA.
    RdaPrfData = 32,

    /// Log data from RDA.
    RdaLogData = 33,

    /// Unknown message type for forward compatibility.
    Unknown(u8),
}
