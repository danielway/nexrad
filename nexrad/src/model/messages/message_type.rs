/// The types of data messages transferred between the RDA and RPG.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
pub enum MessageType {
    /// Replaced by message type 31.
    RDADigitalRadarData = 1,

    /// Metadata.
    RDAStatusData = 2,

    /// Metadata.
    RDAPerformanceMaintenanceData = 3,

    RDAConsoleMessage = 4,

    /// Metadata.
    RDAVolumeCoveragePattern = 5,

    RDAControlCommands = 6,

    RPGVolumeCoveragePattern = 7,

    RPGClutterCensorZones = 8,

    RPGRequestForData = 9,

    RPGConsoleMessage = 10,

    RDALoopBackTest = 11,

    RPGLoopBackTest = 12,

    /// No longer sent.
    RDAClutterFilterBypassMap = 13,

    Spare1 = 14,

    /// Metadata.
    RDAClutterFilterMap = 15,

    ReservedFAARMSOnly1 = 16,

    ReservedFAARMSOnly2 = 17,

    /// Metadata.
    RDAAdaptationData = 18,

    Reserved1 = 20,

    Reserved2 = 21,

    Reserved3 = 22,

    Reserved4 = 23,

    ReservedFAARMSOnly3 = 24,

    ReservedFAARMSOnly4 = 25,

    ReservedFAARMSOnly5 = 26,

    Reserved5 = 29,

    RDADigitalRadarDataGenericFormat = 31,

    RDAPRFData = 32,

    RDALogData = 33,

    Unknown,
}
