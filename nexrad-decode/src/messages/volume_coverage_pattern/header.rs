use super::raw;
use super::{PatternType, PulseWidth};
use crate::messages::VcpNumber;
use std::borrow::Cow;

/// The volume coverage pattern header block
#[derive(Clone, PartialEq, Debug)]
pub struct Header<'a> {
    inner: Cow<'a, raw::Header>,
}

impl<'a> Header<'a> {
    /// Create a new Header wrapper from a raw Header reference.
    pub(crate) fn new(inner: &'a raw::Header) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this header to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Header<'static> {
        Header {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Total message size in halfwords, including the header and all elevation blocks
    pub fn message_size_halfwords(&self) -> u16 {
        self.inner.message_size.get()
    }

    /// Pattern type is always 2
    pub fn pattern_type_raw(&self) -> u16 {
        self.inner.pattern_type.get()
    }

    /// Volume coverage pattern number (raw value). See [`vcp_number()`](Self::vcp_number) for the
    /// decoded value.
    pub fn vcp_number_raw(&self) -> u16 {
        self.inner.pattern_number.get()
    }

    /// Volume coverage pattern number.
    pub fn vcp_number(&self) -> VcpNumber {
        VcpNumber::new(self.inner.pattern_number.get())
    }

    /// Number of elevation cuts in the complete volume scan
    pub fn elevation_cut_count(&self) -> u16 {
        self.inner.elevation_cut_count.get()
    }

    /// Volume Coverage Pattern Version Number
    pub fn version(&self) -> u8 {
        self.inner.version
    }

    /// Clutter map groups are not currently implemented
    pub fn clutter_map_group_number(&self) -> u8 {
        self.inner.clutter_map_group_number
    }

    /// Doppler velocity resolution (raw code). See
    /// [`doppler_velocity_resolution_mps()`](Self::doppler_velocity_resolution_mps) for the
    /// decoded value.
    /// 2 -> 0.5
    /// 4 -> 1.0
    pub fn doppler_velocity_resolution_raw(&self) -> u8 {
        self.inner.doppler_velocity_resolution
    }

    /// Pulse width values.
    /// 2 -> Short
    /// 4 -> Long
    pub fn pulse_width_raw(&self) -> u8 {
        self.inner.pulse_width
    }

    /// VCP sequencing values.
    /// Bits 0-4: Number of Elevations
    /// Bits 5-6: Maximum SAILS Cuts
    /// Bit  13:  Sequence Active
    /// Bit  14:  Truncated VCP
    pub fn vcp_sequencing_raw(&self) -> u16 {
        self.inner.vcp_sequencing.get()
    }

    /// VCP supplemental data.
    /// Bit  0:     SAILS VCP
    /// Bits 1-3:   Number SAILS Cuts
    /// Bit  4:     MRLE VCP
    /// Bits 5-7:   Number MRLE Cuts
    /// Bits 8-10:  Spare
    /// Bit  11:    MPDA VCP
    /// Bit  12:    BASE TILT VCP
    /// Bits 13-15: Number of BASE TILTS
    pub fn vcp_supplemental_data_raw(&self) -> u16 {
        self.inner.vcp_supplemental_data.get()
    }

    /// Pattern type is always 2
    pub fn pattern_type(&self) -> PatternType {
        match self.inner.pattern_type.get() {
            2 => PatternType::Constant,
            other => PatternType::Unknown(other),
        }
    }

    /// Doppler velocity resolution in m/s (0.0 for an unrecognized code).
    pub fn doppler_velocity_resolution_mps(&self) -> f32 {
        match self.inner.doppler_velocity_resolution {
            2 => 0.5,
            4 => 1.0,
            _ => 0.0,
        }
    }

    /// Pulse width values.
    pub fn pulse_width(&self) -> PulseWidth {
        PulseWidth::from_number(self.inner.pulse_width)
    }

    /// Number of elevations from VCP sequencing
    pub fn vcp_sequencing_elevation_count(&self) -> u8 {
        (self.inner.vcp_sequencing.get() & 0x1F) as u8
    }

    /// Maximum SAILS cuts from VCP sequencing
    pub fn vcp_sequencing_max_sails_cuts(&self) -> u8 {
        ((self.inner.vcp_sequencing.get() >> 5) & 0x03) as u8
    }

    /// Sequence active from VCP sequencing
    pub fn vcp_sequencing_sequence_active(&self) -> bool {
        (self.inner.vcp_sequencing.get() >> 13) & 1 == 1
    }

    /// Truncated VCP from VCP sequencing
    pub fn vcp_sequencing_truncated(&self) -> bool {
        (self.inner.vcp_sequencing.get() >> 14) & 1 == 1
    }

    /// Whether this is a SAILS VCP
    pub fn is_sails_vcp(&self) -> bool {
        self.inner.vcp_supplemental_data.get() & 1 == 1
    }

    /// Number of SAILS cuts
    pub fn sails_cut_count(&self) -> u8 {
        ((self.inner.vcp_supplemental_data.get() >> 1) & 0x07) as u8
    }

    /// Whether this is an MRLE VCP
    pub fn is_mrle_vcp(&self) -> bool {
        (self.inner.vcp_supplemental_data.get() >> 4) & 1 == 1
    }

    /// Number of MRLE cuts
    pub fn mrle_cut_count(&self) -> u8 {
        ((self.inner.vcp_supplemental_data.get() >> 5) & 0x07) as u8
    }

    /// Whether this is an MPDA VCP
    pub fn is_mpda_vcp(&self) -> bool {
        (self.inner.vcp_supplemental_data.get() >> 11) & 1 == 1
    }

    /// Whether this is a BASE TILT VCP
    pub fn is_base_tilt_vcp(&self) -> bool {
        (self.inner.vcp_supplemental_data.get() >> 12) & 1 == 1
    }

    /// Number of BASE TILTS
    pub fn base_tilt_count(&self) -> u8 {
        ((self.inner.vcp_supplemental_data.get() >> 13) & 0x07) as u8
    }
}
