use crate::messages::{
    clutter_censor_zones, clutter_filter_bypass_map, clutter_filter_map, console_message,
    digital_radar_data, digital_radar_data_legacy, loopback_test, performance_maintenance_data,
    rda_adaptation_data, rda_control_commands, rda_log_data, rda_prf_data, rda_status_data,
    request_for_data, volume_coverage_pattern, MessageType,
};
use std::borrow::Cow;

/// A decoded NEXRAD Level II message's contents.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MessageContents<'a> {
    /// Message type 2 "RDA Status Data" contains information about the current RDA state, system
    /// control, operating status, scanning strategy, performance parameters like transmitter power
    /// and calibration, and system alarms.
    RdaStatusData(Box<rda_status_data::Message<'a>>),

    /// Message type 3 "Performance/Maintenance Data" contains detailed performance and maintenance
    /// information about the RDA system, including communications, AME, power, transmitter,
    /// tower/utilities, antenna/pedestal, RF generator/receiver, calibration, file status, RSP/CPU
    /// status, and device status.
    PerformanceMaintenanceData(Box<performance_maintenance_data::Message<'a>>),

    /// Message type 1 "Digital Radar Data" is the legacy base data format used from the original
    /// WSR-88D deployment (1991) through Build 10.0 (2008). Each message carries a single radial
    /// of reflectivity, velocity, and/or spectrum width data in fixed 2432-byte frames.
    DigitalRadarDataLegacy(Box<digital_radar_data_legacy::Message<'a>>),

    /// Message type 31 "Digital Radar Data" consists of base data information such as reflectivity,
    /// mean radial velocity, spectrum width, differential reflectivity, differential phase,
    /// correlation coefficient, azimuth angle, elevation angle, cut type, scanning strategy, and
    /// calibration parameters.
    DigitalRadarData(Box<digital_radar_data::Message<'a>>),

    /// Message type 15 "Clutter Filter Map" contains information about clutter filter maps that are
    /// used to filter clutter from radar products
    ClutterFilterMap(Box<clutter_filter_map::Message<'a>>),

    /// Message type 5 "Volume Coverage Pattern" provides details about the volume
    /// coverage pattern being used, including detailed settings for each elevation.
    VolumeCoveragePattern(Box<volume_coverage_pattern::Message<'a>>),

    /// Message types 4 and 10 "Console Message" carry free-form text between the RDA and RPG.
    ConsoleMessage(Box<console_message::Message<'a>>),

    /// Message types 11 and 12 "Loopback Test" are used to test the RDA/RPG wideband interface.
    LoopbackTest(Box<loopback_test::Message<'a>>),

    /// Message type 9 "Request for Data" is sent by the RPG to request specific data from the RDA.
    RequestForData(Box<request_for_data::Message<'a>>),

    /// Message type 6 "RDA Control Commands" contains commands sent from the RPG to control the
    /// RDA system's state, scanning strategy, and various operational parameters.
    RdaControlCommands(Box<rda_control_commands::Message<'a>>),

    /// Message type 8 "Clutter Censor Zones" contains override regions that control clutter
    /// filtering behavior for specific range, azimuth, and elevation zones.
    ClutterCensorZones(Box<clutter_censor_zones::Message<'a>>),

    /// Message type 13 "Clutter Filter Bypass Map" contains information about which range bins
    /// should bypass clutter filtering for each elevation, azimuth, and range bin.
    ClutterFilterBypassMap(Box<clutter_filter_bypass_map::Message<'a>>),

    /// Message type 32 "RDA PRF Data" contains pulse repetition frequency data for each waveform
    /// type used by the radar.
    RdaPrfData(Box<rda_prf_data::Message<'a>>),

    /// Message type 33 "RDA Log Data" contains log file data from the RDA, potentially compressed.
    RdaLogData(Box<rda_log_data::Message<'a>>),

    /// Message type 18 "RDA Adaptation Data" contains site-specific configuration parameters
    /// for the radar system, including antenna parameters, site location, RF path losses,
    /// calibration values, temperature alarm thresholds, and transmitter characteristics.
    RdaAdaptationData(Box<rda_adaptation_data::Message<'a>>),

    /// A message of a type this crate does not decode, such as a reserved type or a type not
    /// carried in Archive II data, with its undecoded payload.
    Other {
        /// The message's type, taken from its header.
        message_type: MessageType,
        /// The message's payload following the message header. For a multi-segment message,
        /// this is each segment's payload concatenated in segment order.
        data: Cow<'a, [u8]>,
    },
}

impl MessageContents<'_> {
    /// The type of the message these contents were decoded from.
    pub fn message_type(&self) -> MessageType {
        match self {
            MessageContents::RdaStatusData(_) => MessageType::RdaStatusData,
            MessageContents::PerformanceMaintenanceData(_) => {
                MessageType::RdaPerformanceMaintenanceData
            }
            MessageContents::DigitalRadarDataLegacy(_) => MessageType::RdaDigitalRadarData,
            MessageContents::DigitalRadarData(_) => MessageType::RdaDigitalRadarDataGenericFormat,
            MessageContents::ClutterFilterMap(_) => MessageType::RdaClutterFilterMap,
            MessageContents::VolumeCoveragePattern(_) => MessageType::RdaVolumeCoveragePattern,
            MessageContents::ConsoleMessage(message) => message.message_type(),
            MessageContents::LoopbackTest(message) => message.message_type(),
            MessageContents::RequestForData(_) => MessageType::RpgRequestForData,
            MessageContents::RdaControlCommands(_) => MessageType::RdaControlCommands,
            MessageContents::ClutterCensorZones(_) => MessageType::RpgClutterCensorZones,
            MessageContents::ClutterFilterBypassMap(_) => MessageType::RdaClutterFilterBypassMap,
            MessageContents::RdaPrfData(_) => MessageType::RdaPrfData,
            MessageContents::RdaLogData(_) => MessageType::RdaLogData,
            MessageContents::RdaAdaptationData(_) => MessageType::RdaAdaptationData,
            MessageContents::Other { message_type, .. } => *message_type,
        }
    }

    /// Convert these contents to an owned version with `'static` lifetime, copying any data
    /// borrowed from the input.
    pub fn into_owned(self) -> MessageContents<'static> {
        match self {
            MessageContents::RdaStatusData(m) => {
                MessageContents::RdaStatusData(Box::new(m.into_owned()))
            }
            MessageContents::PerformanceMaintenanceData(m) => {
                MessageContents::PerformanceMaintenanceData(Box::new(m.into_owned()))
            }
            MessageContents::DigitalRadarDataLegacy(m) => {
                MessageContents::DigitalRadarDataLegacy(Box::new(m.into_owned()))
            }
            MessageContents::DigitalRadarData(m) => {
                MessageContents::DigitalRadarData(Box::new(m.into_owned()))
            }
            MessageContents::ClutterFilterMap(m) => {
                MessageContents::ClutterFilterMap(Box::new(m.into_owned()))
            }
            MessageContents::VolumeCoveragePattern(m) => {
                MessageContents::VolumeCoveragePattern(Box::new(m.into_owned()))
            }
            MessageContents::ConsoleMessage(m) => {
                MessageContents::ConsoleMessage(Box::new(m.into_owned()))
            }
            MessageContents::LoopbackTest(m) => {
                MessageContents::LoopbackTest(Box::new(m.into_owned()))
            }
            MessageContents::RequestForData(m) => {
                MessageContents::RequestForData(Box::new(m.into_owned()))
            }
            MessageContents::RdaControlCommands(m) => {
                MessageContents::RdaControlCommands(Box::new(m.into_owned()))
            }
            MessageContents::ClutterCensorZones(m) => {
                MessageContents::ClutterCensorZones(Box::new(m.into_owned()))
            }
            MessageContents::ClutterFilterBypassMap(m) => {
                MessageContents::ClutterFilterBypassMap(Box::new(m.into_owned()))
            }
            MessageContents::RdaPrfData(m) => MessageContents::RdaPrfData(Box::new(m.into_owned())),
            MessageContents::RdaLogData(m) => MessageContents::RdaLogData(Box::new(m.into_owned())),
            MessageContents::RdaAdaptationData(m) => {
                MessageContents::RdaAdaptationData(Box::new(m.into_owned()))
            }
            MessageContents::Other { message_type, data } => MessageContents::Other {
                message_type,
                data: Cow::Owned(data.into_owned()),
            },
        }
    }
}
