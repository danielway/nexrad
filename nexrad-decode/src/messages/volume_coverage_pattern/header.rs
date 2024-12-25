use serde::Deserialize;

use crate::messages::primitive_aliases::{
    Code1, Code2, Integer1, Integer2, Integer4,
};

use std::fmt::Debug;

use crate::messages::volume_coverage_pattern::definitions::*;

#[cfg(feature = "uom")]
use uom::si::f64::Velocity;
#[cfg(feature = "uom")]
use uom::si::velocity::meter_per_second;

/// The volume coverage pattern header block which contains information about the volume coverage pattern
/// and the following data for each elevation
#[derive(Clone, PartialEq, Deserialize)]
pub struct Header {
    /// Total message size in halfwords, including the header and all elevation blocks
    pub message_size: Integer2,

    /// Pattern type is always 2
    pub pattern_type: Code2,

    /// VCP Pattern Number
    pub pattern_number: Integer2,

    /// Number of elevation cuts in one complete volume scan
    pub number_of_elevation_cuts: Integer2,

    /// VCP version number
    pub version: Integer1,

    /// Clutter map groups are not currently implemented
    pub clutter_map_group_number: Integer1,

    /// Doppler velocity resolution.
    /// 2 -> 0.5
    /// 4 -> 1.0
    pub doppler_velocity_resolution: Code1,

    /// Pulse width values.
    /// 2 -> Short
    /// 4 -> Long
    pub pulse_width: Code1,

    /// Reserved
    pub reserved_1: Integer4,

    /// VCP sequencing values.
    /// Bits 0-4: Number of Elevations
    /// Bits 5-6: Maximum SAILS Cuts
    /// Bit  13:  Sequence Active
    /// Bit  14:  Truncated VCP
    pub vcp_sequencing: Code2,

    /// VCP supplemental data.
    /// Bit  0:     SAILS VCP
    /// Bits 1-3:   Number SAILS Cuts
    /// Bit  4:     MRLE VCP
    /// Bits 5-7:   Number MRLE Cuts
    /// Bits 8-10:  Spare
    /// Bit  11:    MPDA VCP
    /// Bit  12:    BASE TILT VCP
    /// Bits 13-15: Number of BASE TILTS
    pub vcp_supplemental_data: Code2,

    /// Reserved
    pub reserved_2: Integer2,
}

impl Header {
    /// The pattern type of the volume coverage pattern
    pub fn pattern_type(&self) -> PatternType {
        match self.pattern_type {
            2 => PatternType::Constant,
            _ => PatternType::Unknown,
        }
    }

    #[cfg(feature = "uom")]
    pub fn doppler_velocity_resolution(&self) -> Option<Velocity> {
        match self.doppler_velocity_resolution {
            2 => Some(Velocity::new::<meter_per_second>(0.5)),
            4 => Some(Velocity::new::<meter_per_second>(1.0)),
            _ => None,
        }
    }

    pub fn pulse_width(&self) -> PulseWidth {
        match self.pulse_width {
            2 => PulseWidth::Short,
            4 => PulseWidth::Long,
            _ => PulseWidth::Unknown,
        }
    }

    pub fn vcp_sequencing_number_of_elevations(&self) -> u8 {
        (self.vcp_sequencing & 0x001F) as u8
    }

    pub fn vcp_sequencing_maximum_sails_cuts(&self) -> u8 {
        ((self.vcp_sequencing & 0x0060) >> 5) as u8
    }

    pub fn vcp_sequencing_sequence_active(&self) -> bool {
        ((self.vcp_sequencing & 0x2000) >> 13) == 1
    }

    pub fn vcp_sequencing_truncated_vcp(&self) -> bool {
        ((self.vcp_sequencing & 0x4000) >> 14) == 1
    }

    pub fn vcp_supplemental_data_sails_vcp(&self) -> bool {
        (self.vcp_supplemental_data & 0x0001) == 1
    }

    pub fn vcp_supplemental_data_number_sails_cuts(&self) -> u8 {
        ((self.vcp_supplemental_data & 0x000E) >> 1) as u8
    }

    pub fn vcp_supplemental_data_mrle_vcp(&self) -> bool {
        ((self.vcp_supplemental_data & 0x0010) >> 4) == 1
    }

    pub fn vcp_supplemental_data_number_mrle_cuts(&self) -> u8 {
        ((self.vcp_supplemental_data & 0x00E0) >> 5) as u8
    }

    pub fn vcp_supplemental_data_mpda_vcp(&self) -> bool {
        ((self.vcp_supplemental_data & 0x0800) >> 11) == 1
    }

    pub fn vcp_supplemental_data_base_tilt_vcp(&self) -> bool {
        ((self.vcp_supplemental_data & 0x1000) >> 12) == 1
    }

    pub fn vcp_supplemental_data_base_tilts(&self) -> u8 {
        ((self.vcp_supplemental_data & 0xE000) >> 13) as u8
    }
}

#[cfg(not(feature = "uom"))]
impl Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("message_size", &self.message_size)
            .field("pattern_type", &self.pattern_type())
            .field("pattern_number", &self.pattern_number)
            .field("number_of_elevation_cuts", &self.number_of_elevation_cuts)
            .field("version", &self.version)
            .field("clutter_map_group_number", &self.clutter_map_group_number)
            .field("doppler_velocity_resolution", &self.doppler_velocity_resolution)
            .field("pulse_width", &self.pulse_width())
            .field("reserved_1", &self.reserved_1)
            .field("vcp_sequencing_raw", &self.vcp_sequencing)
            .field("vcp_sequencing_number_of_elevations", &self.vcp_sequencing_number_of_elevations())
            .field("vcp_sequencing_maximum_sails_cuts", &self.vcp_sequencing_maximum_sails_cuts())
            .field("vcp_sequencing_sequence_active", &self.vcp_sequencing_sequence_active())
            .field("vcp_sequencing_truncated_vcp", &self.vcp_sequencing_truncated_vcp())
            .field("vcp_supplemental_data_raw", &self.vcp_supplemental_data)
            .field("vcp_supplemental_data_sails_vcp", &self.vcp_supplemental_data_sails_vcp())
            .field("vcp_supplemental_data_number_sails_cuts", &self.vcp_supplemental_data_number_sails_cuts())
            .field("vcp_supplemental_data_mrle_vcp", &self.vcp_supplemental_data_mrle_vcp())
            .field("vcp_supplemental_data_number_mrle_cuts", &self.vcp_supplemental_data_number_mrle_cuts())
            .field("vcp_supplemental_data_mpda_vcp", &self.vcp_supplemental_data_mpda_vcp())
            .field("vcp_supplemental_data_base_tilt_vcp", &self.vcp_supplemental_data_base_tilt_vcp())
            .field("vcp_supplemental_data_base_tilts", &self.vcp_supplemental_data_base_tilts())
            .field("reserved_2", &self.reserved_2)
            .finish()
    }
}

#[cfg(feature = "uom")]
impl Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("message_size", &self.message_size)
            .field("pattern_type", &self.pattern_type())
            .field("pattern_number", &self.pattern_number)
            .field("number_of_elevation_cuts", &self.number_of_elevation_cuts)
            .field("version", &self.version)
            .field("clutter_map_group_number", &self.clutter_map_group_number)
            .field("doppler_velocity_resolution", &self.doppler_velocity_resolution())
            .field("pulse_width", &self.pulse_width())
            .field("reserved_1", &self.reserved_1)
            .field("vcp_sequencing_raw", &self.vcp_sequencing)
            .field("vcp_sequencing_number_of_elevations", &self.vcp_sequencing_number_of_elevations())
            .field("vcp_sequencing_maximum_sails_cuts", &self.vcp_sequencing_maximum_sails_cuts())
            .field("vcp_sequencing_sequence_active", &self.vcp_sequencing_sequence_active())
            .field("vcp_sequencing_truncated_vcp", &self.vcp_sequencing_truncated_vcp())
            .field("vcp_supplemental_data_raw", &self.vcp_supplemental_data)
            .field("vcp_supplemental_data_sails_vcp", &self.vcp_supplemental_data_sails_vcp())
            .field("vcp_supplemental_data_number_sails_cuts", &self.vcp_supplemental_data_number_sails_cuts())
            .field("vcp_supplemental_data_mrle_vcp", &self.vcp_supplemental_data_mrle_vcp())
            .field("vcp_supplemental_data_number_mrle_cuts", &self.vcp_supplemental_data_number_mrle_cuts())
            .field("vcp_supplemental_data_mpda_vcp", &self.vcp_supplemental_data_mpda_vcp())
            .field("vcp_supplemental_data_base_tilt_vcp", &self.vcp_supplemental_data_base_tilt_vcp())
            .field("vcp_supplemental_data_base_tilts", &self.vcp_supplemental_data_base_tilts())
            .field("reserved_2", &self.reserved_2)
            .finish()
    }
}
