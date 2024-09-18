use crate::messages::primitive_aliases::Code2;
use std::fmt::Debug;

/// The multiple flags for the RDA system's scan and data status.
pub struct ScanDataFlags(Code2);

impl ScanDataFlags {
    pub(crate) fn new(value: Code2) -> Self {
        Self(value)
    }

    /// Whether AVSET is enabled.
    pub fn avset_enabled(&self) -> bool {
        let enabled_flag = self.0 & 0b0001 != 0;
        let disabled_flag = self.0 & 0b0010 != 0;
        debug_assert!(
            enabled_flag ^ disabled_flag,
            "Unexpected AVSET state (expected: enabled XOR disabled)"
        );
        enabled_flag
    }

    /// Whether EBC is enabled.
    pub fn ebc_enabled(&self) -> bool {
        self.0 & 0b0100 != 0
    }

    /// Whether RDA log data is enabled.
    pub fn rda_log_data_enabled(&self) -> bool {
        self.0 & 0b1000 != 0
    }

    /// Whether time series data recording is enabled.
    pub fn time_series_data_recording_enabled(&self) -> bool {
        self.0 & 0b10000 != 0
    }
}

impl Debug for ScanDataFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScanDataFlags")
            .field("avset_enabled", &self.avset_enabled())
            .field("ebc_enabled", &self.ebc_enabled())
            .field("rda_log_data_enabled", &self.rda_log_data_enabled())
            .field(
                "time_series_data_recording_enabled",
                &self.time_series_data_recording_enabled(),
            )
            .finish()
    }
}
