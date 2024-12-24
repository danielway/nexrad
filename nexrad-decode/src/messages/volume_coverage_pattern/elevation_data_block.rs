use serde::Deserialize;

use crate::messages::primitive_aliases::{Code1, Code2, Integer1, Integer2, ScaledSInteger2};

use std::fmt::Debug;

/// A radial data moment block.
#[derive(Clone, PartialEq, Deserialize)]
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
    pub waveform_types: Code1,

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
    pub vcp_supplemental_data: Code2,

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


impl Debug for ElevationDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElevationDataBlock")
            .field("elevation_angle", &self.elevation_angle)
            .field("channel_configuration", &self.channel_configuration)
            .field("waveform_types", &self.waveform_types)
            .field("super_resolution_control", &self.super_resolution_control)
            .field("surveillance_prf_number", &self.surveillance_prf_number)
            .field("surveillance_prf_pulse_count_radial", &self.surveillance_prf_pulse_count_radial)
            .field("azimuth_rate", &self.azimuth_rate)
            .field("reflectivity_threshold", &self.reflectivity_threshold)
            .field("velocity_threshold", &self.velocity_threshold)
            .field("spectrum_width_threshold", &self.spectrum_width_threshold)
            .field("differential_reflectivity_threshold", &self.differential_reflectivity_threshold)
            .field("differential_phase_threshold", &self.differential_phase_threshold)
            .field("correlation_coefficient_threshold", &self.correlation_coefficient_threshold)
            .field("sector_1_edge_angle", &self.sector_1_edge_angle)
            .field("sector_1_doppler_prf_number", &self.sector_1_doppler_prf_number)
            .field("sector_1_doppler_prf_pulse_count_radial", &self.sector_1_doppler_prf_pulse_count_radial)
            .field("vcp_supplemental_data", &self.vcp_supplemental_data)
            .field("sector_2_edge_angle", &self.sector_2_edge_angle)
            .field("sector_2_doppler_prf_number", &self.sector_2_doppler_prf_number)
            .field("sector_2_doppler_prf_pulse_count_radial", &self.sector_2_doppler_prf_pulse_count_radial)
            .field("ebc_angle", &self.ebc_angle)
            .field("sector_3_edge_angle", &self.sector_3_edge_angle)
            .field("sector_3_doppler_prf_number", &self.sector_3_doppler_prf_number)
            .field("sector_3_doppler_prf_pulse_count_radial", &self.sector_3_doppler_prf_pulse_count_radial)
            .field("reserved", &self.reserved)
            .finish()
    }
}