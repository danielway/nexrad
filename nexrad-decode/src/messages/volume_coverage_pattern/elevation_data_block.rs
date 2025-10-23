use std::fmt::Debug;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use crate::messages::primitive_aliases::{Code1, Code2, Integer1, Integer2, ScaledSInteger2};
use crate::messages::volume_coverage_pattern::definitions::{ChannelConfiguration, WaveformType};

#[cfg(feature = "uom")]
use uom::si::{
    angle::degree,
    angular_velocity::degree_per_second,
    f64::{Angle, AngularVelocity},
};

/// A data block for a single elevation cut.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, TryFromBytes, Immutable, KnownLayout)]
pub struct ElevationDataBlock {
    /// The elevation angle for this cut
    pub elevation_angle: Code2,

    /// The channel configuration for this cut
    /// 0 => Constant Phase
    /// 1 => Random Phase
    /// 2 => SZ2 Phase
    pub channel_configuration: Code1,

    /// The waveform type for this cut
    /// 1 => Contiguous Surveillance
    /// 2 => Contiguous Doppler w/ Ambiguity Resolution
    /// 3 => Contiguous Doppler w/o Ambiguity Resolution
    /// 4 => Batch
    /// 5 => Staggered Pulse Pair
    pub waveform_type: Code1,

    /// Super resolution control values for this cut
    /// Bit 0: 0.5 degree azimuth
    /// Bit 1: 1/4 km reflectivity
    /// Bit 2: Doppler to 300 km
    /// Bit 3: Dual polarization to 300 km
    pub super_resolution_control: Code1,

    /// The pulse repetition frequency number for surveillance cuts
    pub surveillance_prf_number: Integer1,

    /// The pulse count per radial for surveillance cuts
    pub surveillance_prf_pulse_count_radial: Integer2,

    /// The azimuth rate of the cut
    pub azimuth_rate: Code2,

    /// Signal to noise ratio (SNR) threshold for reflectivity
    pub reflectivity_threshold: ScaledSInteger2,

    /// Signal to noise ratio (SNR) threshold for velocity
    pub velocity_threshold: ScaledSInteger2,

    /// Signal to noise ratio (SNR) threshold for spectrum width
    pub spectrum_width_threshold: ScaledSInteger2,

    /// Signal to noise ratio (SNR) threshold for differential reflectivity
    pub differential_reflectivity_threshold: ScaledSInteger2,

    /// Signal to noise ratio (SNR) threshold for differential phase
    pub differential_phase_threshold: ScaledSInteger2,

    /// Signal to noise ratio (SNR) threshold for correlation coefficitn
    pub correlation_coefficient_threshold: ScaledSInteger2,

    /// Sector 1 Azimuth Clockwise Edge Angle (denotes start angle)
    pub sector_1_edge_angle: Code2,

    /// Sector 1 Doppler PRF Number
    pub sector_1_doppler_prf_number: Integer2,

    /// Sector 1 Doppler Pulse Count/Radial
    pub sector_1_doppler_prf_pulse_count_radial: Integer2,

    /// Supplemental Data
    /// Bit 0:    SAILS Cut
    /// Bits 1-3: SAILS Sequence Number
    /// Bit 4:    MRLE Cut
    /// Bits 5-7: MRLE Sequence Number
    /// Bit 8:    Spare
    /// Bit 9:    MPDA Cut
    /// Bit 10:   BASE TILT Cut
    pub supplemental_data: Code2,

    /// Sector 2 Azimuth Clockwise Edge Angle (denotes start angle)
    pub sector_2_edge_angle: Code2,

    /// Sector 2 Doppler PRF Number
    pub sector_2_doppler_prf_number: Integer2,

    /// Sector 2 Doppler Pulse Count/Radial
    pub sector_2_doppler_prf_pulse_count_radial: Integer2,

    /// The correction added to the elevation angle for this cut
    pub ebc_angle: Code2,

    /// Sector 3 Azimuth Clockwise Edge Angle (denotes start angle)
    pub sector_3_edge_angle: Code2,

    /// Sector 3 Doppler PRF Number
    pub sector_3_doppler_prf_number: Integer2,

    /// Sector 3 Doppler Pulse Count/Radial
    pub sector_3_doppler_prf_pulse_count_radial: Integer2,

    /// Reserved
    pub reserved: Integer2,
}

/// Decodes an angle as defined in table III-A of ICD 2620002W
fn decode_angle(raw: &Code2) -> f64 {
    let raw = raw.get();
    let mut angle: f64 = 0.0;
    for i in 3..16 {
        if ((raw >> i) & 1) == 1 {
            angle += 180.0 * f64::powf(2.0, (i - 15) as f64);
        }
    }

    angle
}

/// Decodes an angular velocity as defined in table XI-D of ICD 2620002W
fn decode_angular_velocity(raw: &Code2) -> f64 {
    let raw = raw.get();
    let mut angular_velocity: f64 = 0.0;

    for i in 3..15 {
        if ((raw >> i) & 1) == 1 {
            angular_velocity += 22.5 * f64::powf(2.0, (i - 14) as f64);
        }
    }

    if ((raw >> 15) & 1) == 1 {
        angular_velocity = -angular_velocity
    }

    angular_velocity
}

impl ElevationDataBlock {
    /// Decodes a reference to a ElevationDataBlock from a byte slice, returning the block and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a ElevationDataBlock from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (block, _) = Self::decode_ref(bytes)?;
        Ok(block.clone())
    }

    /// The elevation angle for this cut
    #[cfg(feature = "uom")]
    pub fn elevation_angle(&self) -> Angle {
        Angle::new::<degree>(decode_angle(&self.elevation_angle))
    }

    /// The elevation angle for this cut, in degrees
    pub fn elevation_angle_degrees(&self) -> f64 {
        decode_angle(&self.elevation_angle)
    }

    /// The channel configuration for this cut
    pub fn channel_configuration(&self) -> ChannelConfiguration {
        match self.channel_configuration {
            0 => ChannelConfiguration::ConstantPhase,
            1 => ChannelConfiguration::RandomPhase,
            2 => ChannelConfiguration::SZ2Phase,
            _ => ChannelConfiguration::UnknownPhase,
        }
    }

    /// The waveform type for this cut
    pub fn waveform_type(&self) -> WaveformType {
        match self.waveform_type {
            1 => WaveformType::CS,
            2 => WaveformType::CDW,
            3 => WaveformType::CDWO,
            4 => WaveformType::B,
            5 => WaveformType::SPP,
            _ => WaveformType::Unknown,
        }
    }

    /// Whether this cut uses super resolution 0.5 degree azimuth
    pub fn super_resolution_control_half_degree_azimuth(&self) -> bool {
        (self.super_resolution_control & 0x1) == 1
    }

    /// Whether this cut uses super resolution 0.25 km reflectivity
    pub fn super_resolution_control_quarter_km_reflectivity(&self) -> bool {
        ((self.super_resolution_control >> 1) & 0x1) == 1
    }

    /// Whether this cut uses super resolution doppler to 300 km
    pub fn super_resolution_control_doppler_to_300km(&self) -> bool {
        ((self.super_resolution_control >> 2) & 0x1) == 1
    }

    /// Whether this cut uses super resolution dual polarization to 300km
    pub fn super_resolution_control_dual_polarization_to_300km(&self) -> bool {
        ((self.super_resolution_control >> 3) & 0x1) == 1
    }

    /// The azimuth rate used for this cut
    #[cfg(feature = "uom")]
    pub fn azimuth_rate(&self) -> AngularVelocity {
        AngularVelocity::new::<degree_per_second>(decode_angular_velocity(&self.azimuth_rate))
    }

    /// The azimuth rate used for this cut, in degrees per second
    pub fn azimuth_rate_degrees_per_second(&self) -> f64 {
        decode_angular_velocity(&self.azimuth_rate)
    }

    /// The reflectivity threshold for this cut
    pub fn reflectivity_threshold(&self) -> f64 {
        self.reflectivity_threshold.get() as f64 * 0.125
    }

    /// The velocity threshold for this cut
    pub fn velocity_threshold(&self) -> f64 {
        self.velocity_threshold.get() as f64 * 0.125
    }

    /// The spectrum width threshold for this cut
    pub fn spectrum_width_threshold(&self) -> f64 {
        self.spectrum_width_threshold.get() as f64 * 0.125
    }

    /// The differential reflectivity threshold for this cut
    pub fn differential_reflectivity_threshold(&self) -> f64 {
        self.differential_reflectivity_threshold.get() as f64 * 0.125
    }

    /// The differential phase threshold for this cut
    pub fn differential_phase_threshold(&self) -> f64 {
        self.differential_phase_threshold.get() as f64 * 0.125
    }

    /// The correlation coefficient threshold for this cut
    pub fn correlation_coefficient_threshold(&self) -> f64 {
        self.correlation_coefficient_threshold.get() as f64 * 0.125
    }

    /// Sector 1 Azimuth Clockwise Edge Angle (denotes start angle)
    #[cfg(feature = "uom")]
    pub fn sector_1_edge_angle(&self) -> Angle {
        Angle::new::<degree>(decode_angle(&self.sector_1_edge_angle))
    }

    /// Sector 1 Azimuth Clockwise Edge Angle (denotes start angle), in degrees
    pub fn sector_1_edge_angle_degrees(&self) -> f64 {
        decode_angle(&self.sector_1_edge_angle)
    }

    /// Sector 2 Azimuth Clockwise Edge Angle (denotes start angle)
    #[cfg(feature = "uom")]
    pub fn sector_2_edge_angle(&self) -> Angle {
        Angle::new::<degree>(decode_angle(&self.sector_2_edge_angle))
    }

    /// Sector 2 Azimuth Clockwise Edge Angle (denotes start angle), in degrees
    pub fn sector_2_edge_angle_degrees(&self) -> f64 {
        decode_angle(&self.sector_2_edge_angle)
    }

    /// Sector 3 Azimuth Clockwise Edge Angle (denotes start angle)
    #[cfg(feature = "uom")]
    pub fn sector_3_edge_angle(&self) -> Angle {
        Angle::new::<degree>(decode_angle(&self.sector_3_edge_angle))
    }

    /// Sector 3 Azimuth Clockwise Edge Angle (denotes start angle), in degrees
    pub fn sector_3_edge_angle_degrees(&self) -> f64 {
        decode_angle(&self.sector_3_edge_angle)
    }

    /// The correction added to the elevation angle for this cut
    #[cfg(feature = "uom")]
    pub fn ebc_angle(&self) -> Angle {
        Angle::new::<degree>(decode_angle(&self.ebc_angle))
    }

    /// The correction added to the elevation angle for this cut, in degrees
    pub fn ebc_angle_degrees(&self) -> f64 {
        decode_angle(&self.ebc_angle)
    }

    /// Whether this cut is a SAILS cut
    pub fn supplemental_data_sails_cut(&self) -> bool {
        (self.supplemental_data.get() & 0x0001) == 1
    }

    /// The SAILS sequence number of this cut
    pub fn supplemental_data_sails_sequence_number(&self) -> u8 {
        ((self.supplemental_data.get() & 0x000E) >> 1) as u8
    }

    /// Whether this cut is an MRLE cut
    pub fn supplemental_data_mrle_cut(&self) -> bool {
        ((self.supplemental_data.get() & 0x0010) >> 4) == 1
    }

    /// The MRLE sequence number of this cut
    pub fn supplemental_data_mrle_sequence_number(&self) -> u8 {
        ((self.supplemental_data.get() & 0x00E0) >> 5) as u8
    }

    /// Whether this cut is an MPDA cut
    pub fn supplemental_data_mpda_cut(&self) -> bool {
        ((self.supplemental_data.get() & 0x0200) >> 9) == 1
    }

    /// Whether this cut is a BASE TILT cut
    pub fn supplemental_data_base_tilt_cut(&self) -> bool {
        ((self.supplemental_data.get() & 0x0400) >> 10) == 1
    }
}
