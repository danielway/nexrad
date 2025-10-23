use std::fmt::Debug;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

use crate::messages::primitive_aliases::{Code1, Code2, Integer1, Integer2, Integer4};
use crate::messages::volume_coverage_pattern::definitions::*;

#[cfg(feature = "uom")]
use uom::si::{f64::Velocity, velocity::meter_per_second};

/// The volume coverage pattern header block
#[repr(C)]
#[derive(Clone, PartialEq, Debug, TryFromBytes, Immutable, KnownLayout)]
pub struct Header {
    /// Total message size in halfwords, including the header and all elevation blocks
    pub message_size: Integer2,

    /// Pattern type is always 2
    pub pattern_type: Code2,

    /// Volume Coverage Pattern Number
    pub pattern_number: Integer2,

    /// Number of elevation cuts in the complete volume scan
    pub number_of_elevation_cuts: Integer2,

    /// Volume Coverage Pattern Version Number
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
    /// Decodes a reference to a Header from a byte slice, returning the header and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a Header from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (header, _) = Self::decode_ref(bytes)?;
        Ok(header.clone())
    }

    /// The pattern type of the volume coverage pattern
    pub fn pattern_type(&self) -> PatternType {
        match self.pattern_type.get() {
            2 => PatternType::Constant,
            _ => PatternType::Unknown,
        }
    }

    /// The doppler velocity resolution of this coverage pattern
    #[cfg(feature = "uom")]
    pub fn doppler_velocity_resolution(&self) -> Option<Velocity> {
        match self.doppler_velocity_resolution {
            2 => Some(Velocity::new::<meter_per_second>(0.5)),
            4 => Some(Velocity::new::<meter_per_second>(1.0)),
            _ => None,
        }
    }

    /// The doppler velocity resolution of this coverage pattern in m/s
    pub fn doppler_velocity_resolution_meters_per_second(&self) -> Option<f64> {
        match self.doppler_velocity_resolution {
            2 => Some(0.5),
            4 => Some(1.0),
            _ => None,
        }
    }

    /// The pulse width for this VCP
    pub fn pulse_width(&self) -> PulseWidth {
        match self.pulse_width {
            2 => PulseWidth::Short,
            4 => PulseWidth::Long,
            _ => PulseWidth::Unknown,
        }
    }

    /// The number of elevations in the VCP
    pub fn vcp_sequencing_number_of_elevations(&self) -> u8 {
        (self.vcp_sequencing.get() & 0x001F) as u8
    }

    /// The maximum number of SAILS cuts allowed in this VCP
    pub fn vcp_sequencing_maximum_sails_cuts(&self) -> u8 {
        ((self.vcp_sequencing.get() & 0x0060) >> 5) as u8
    }

    /// Whether this VCP is a part of an active VCP sequence
    pub fn vcp_sequencing_sequence_active(&self) -> bool {
        ((self.vcp_sequencing.get() & 0x2000) >> 13) == 1
    }

    /// Whether this VCP is truncated
    pub fn vcp_sequencing_truncated_vcp(&self) -> bool {
        ((self.vcp_sequencing.get() & 0x4000) >> 14) == 1
    }

    /// Whether this VCP uses SAILS cuts
    pub fn vcp_supplemental_data_sails_vcp(&self) -> bool {
        (self.vcp_supplemental_data.get() & 0x0001) == 1
    }

    /// The number of SAILS cuts used by this VCP
    pub fn vcp_supplemental_data_number_sails_cuts(&self) -> u8 {
        ((self.vcp_supplemental_data.get() & 0x000E) >> 1) as u8
    }

    /// Whether this VCP uses MRLE cuts
    pub fn vcp_supplemental_data_mrle_vcp(&self) -> bool {
        ((self.vcp_supplemental_data.get() & 0x0010) >> 4) == 1
    }

    /// The number of MRLE cuts used by this VCP
    pub fn vcp_supplemental_data_number_mrle_cuts(&self) -> u8 {
        ((self.vcp_supplemental_data.get() & 0x00E0) >> 5) as u8
    }

    /// Whether this VCP is a Multi-PRF Dealiasing Algorithm (MPDA) VCP
    pub fn vcp_supplemental_data_mpda_vcp(&self) -> bool {
        ((self.vcp_supplemental_data.get() & 0x0800) >> 11) == 1
    }

    /// Whether this VCP contains BASE TILTS
    pub fn vcp_supplemental_data_base_tilt_vcp(&self) -> bool {
        ((self.vcp_supplemental_data.get() & 0x1000) >> 12) == 1
    }

    /// The number of BASE TILTS in this VCP
    pub fn vcp_supplemental_data_base_tilts(&self) -> u8 {
        ((self.vcp_supplemental_data.get() & 0xE000) >> 13) as u8
    }
}
