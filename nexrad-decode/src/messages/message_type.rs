/// The types of data messages transferred between the RDA and RPG.
///
/// Use [`MessageType::from_number`] to classify a message type number as it appears in the
/// message header, and [`MessageType::number`] to recover it.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
#[non_exhaustive]
pub enum MessageType {
    /// Message type 1. Replaced by message type 31.
    DigitalRadarDataLegacy,

    /// Message type 2. Metadata.
    RdaStatusData,

    /// Message type 3. Metadata.
    PerformanceMaintenanceData,

    /// Message type 4. Console message from RDA.
    RdaConsoleMessage,

    /// Message type 5. Metadata.
    RdaVolumeCoveragePattern,

    /// Message type 6. Control commands to RDA.
    RdaControlCommands,

    /// Message type 7. VCP from RPG.
    RpgVolumeCoveragePattern,

    /// Message type 8. Clutter censor zones from RPG.
    ClutterCensorZones,

    /// Message type 9. Data request from RPG.
    RequestForData,

    /// Message type 10. Console message from RPG.
    RpgConsoleMessage,

    /// Message type 11. Loopback test from RDA.
    RdaLoopbackTest,

    /// Message type 12. Loopback test from RPG.
    RpgLoopbackTest,

    /// Message type 13. No longer sent.
    ClutterFilterBypassMap,

    /// Message type 15. Metadata.
    ClutterFilterMap,

    /// Message type 18. Metadata.
    RdaAdaptationData,

    /// Message type 29. Model Data, sent to the RPG from AWIPS and written to the Archive II
    /// interface after a volume (Archive II ICD 2620010J §7.6.7). The payload is encoded with
    /// External Data Representation (XDR) and is not decoded by this crate: its contents decode as
    /// [`MessageContents::Other`](crate::messages::MessageContents::Other). In the archive bucket
    /// it is stored as a separate `_MDM` object.
    ModelData,

    /// Message type 31. Generic format digital radar data (primary data format).
    DigitalRadarData,

    /// Message type 32. PRF data from RDA.
    RdaPrfData,

    /// Message type 33. Log data from RDA.
    RdaLogData,

    /// A message type number the ICD designates as spare or reserved, carrying that number.
    ///
    /// - 14 is spare.
    /// - 16, 17, 24, 25 and 26 are reserved for FAA RMS use.
    /// - 20, 21, 22 and 23 are reserved.
    Reserved(u8),

    /// A message type number the ICD does not define, carrying that number. Kept for forward
    /// compatibility.
    Unknown(u8),
}

impl MessageType {
    /// Classify a message type number as it appears in the message header.
    ///
    /// Numbers the ICD designates as spare or reserved map to [`MessageType::Reserved`]; numbers
    /// the ICD does not define map to [`MessageType::Unknown`].
    pub fn from_number(number: u8) -> Self {
        match number {
            1 => MessageType::DigitalRadarDataLegacy,
            2 => MessageType::RdaStatusData,
            3 => MessageType::PerformanceMaintenanceData,
            4 => MessageType::RdaConsoleMessage,
            5 => MessageType::RdaVolumeCoveragePattern,
            6 => MessageType::RdaControlCommands,
            7 => MessageType::RpgVolumeCoveragePattern,
            8 => MessageType::ClutterCensorZones,
            9 => MessageType::RequestForData,
            10 => MessageType::RpgConsoleMessage,
            11 => MessageType::RdaLoopbackTest,
            12 => MessageType::RpgLoopbackTest,
            13 => MessageType::ClutterFilterBypassMap,
            15 => MessageType::ClutterFilterMap,
            18 => MessageType::RdaAdaptationData,
            29 => MessageType::ModelData,
            31 => MessageType::DigitalRadarData,
            32 => MessageType::RdaPrfData,
            33 => MessageType::RdaLogData,
            14 | 16 | 17 | 20..=26 => MessageType::Reserved(number),
            other => MessageType::Unknown(other),
        }
    }

    /// The message type number as it appears in the message header.
    pub fn number(self) -> u8 {
        match self {
            MessageType::DigitalRadarDataLegacy => 1,
            MessageType::RdaStatusData => 2,
            MessageType::PerformanceMaintenanceData => 3,
            MessageType::RdaConsoleMessage => 4,
            MessageType::RdaVolumeCoveragePattern => 5,
            MessageType::RdaControlCommands => 6,
            MessageType::RpgVolumeCoveragePattern => 7,
            MessageType::ClutterCensorZones => 8,
            MessageType::RequestForData => 9,
            MessageType::RpgConsoleMessage => 10,
            MessageType::RdaLoopbackTest => 11,
            MessageType::RpgLoopbackTest => 12,
            MessageType::ClutterFilterBypassMap => 13,
            MessageType::ClutterFilterMap => 15,
            MessageType::RdaAdaptationData => 18,
            MessageType::ModelData => 29,
            MessageType::DigitalRadarData => 31,
            MessageType::RdaPrfData => 32,
            MessageType::RdaLogData => 33,
            MessageType::Reserved(number) | MessageType::Unknown(number) => number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MessageType;

    #[test]
    fn number_round_trips_for_every_value() {
        for number in 0..=u8::MAX {
            assert_eq!(MessageType::from_number(number).number(), number);
        }
    }

    #[test]
    fn model_data_is_message_type_29() {
        assert_eq!(MessageType::from_number(29), MessageType::ModelData);
        assert_eq!(MessageType::ModelData.number(), 29);
    }

    #[test]
    fn spare_and_reserved_numbers_classify_as_reserved() {
        for number in [14, 16, 17, 20, 21, 22, 23, 24, 25, 26] {
            assert_eq!(
                MessageType::from_number(number),
                MessageType::Reserved(number)
            );
        }
        for number in [0, 19, 27, 28, 30, 34, 255] {
            assert_eq!(
                MessageType::from_number(number),
                MessageType::Unknown(number)
            );
        }
    }
}
