use crate::binary_data::BinaryData;
use crate::messages::primitive_aliases::{Integer1, Integer2, Real4, SInteger2};
use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// Legacy volume data moment block (Build 19.0 and earlier, 40 bytes).
///
/// This format was used in NEXRAD builds through 20.0 and does not include
/// the ZDR bias estimate fields added in later builds.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, FromBytes, Immutable, KnownLayout)]
pub struct VolumeDataBlockLegacy {
    /// Size of data block in bytes.
    pub lrtup: Integer2,

    /// Major version number.
    pub major_version_number: Integer1,

    /// Minor version number.
    pub minor_version_number: Integer1,

    /// Latitude of radar in degrees.
    pub latitude: Real4,

    /// Longitude of radar in degrees.
    pub longitude: Real4,

    /// Height of site base above sea level in meters.
    pub site_height: SInteger2,

    /// Height of radar tower above ground in meters.
    pub tower_height: Integer2,

    /// Reflectivity scaling factor without correction by ground noise scaling factors given in
    /// adaptation data message in dB.
    pub calibration_constant: Real4,

    /// Transmitter power for horizontal channel in kW.
    pub horizontal_shv_tx_power: Real4,

    /// Transmitter power for vertical channel in kW.
    pub vertical_shv_tx_power: Real4,

    /// Calibration of system ZDR in dB.
    pub system_differential_reflectivity: Real4,

    /// Initial DP for the system in degrees.
    pub initial_system_differential_phase: Real4,

    /// Identifies the volume coverage pattern in use.
    pub volume_coverage_pattern_number: Integer2,

    /// Processing option flags.
    ///
    /// Options:
    ///   0 = RxR noise
    ///   1 = CBT
    pub processing_status: Integer2,
}

/// A volume data moment block (Build 18.0 and later, 48 bytes).
#[repr(C)]
#[derive(Clone, PartialEq, Debug, FromBytes, Immutable, KnownLayout)]
pub struct VolumeDataBlock {
    /// Size of data block in bytes.
    pub lrtup: Integer2,

    /// Major version number.
    pub major_version_number: Integer1,

    /// Minor version number.
    pub minor_version_number: Integer1,

    /// Latitude of radar in degrees.
    pub latitude: Real4,

    /// Longitude of radar in degrees.
    pub longitude: Real4,

    /// Height of site base above sea level in meters.
    pub site_height: SInteger2,

    /// Height of radar tower above ground in meters.
    pub tower_height: Integer2,

    /// Reflectivity scaling factor without correction by ground noise scaling factors given in
    /// adaptation data message in dB.
    pub calibration_constant: Real4,

    /// Transmitter power for horizontal channel in kW.
    pub horizontal_shv_tx_power: Real4,

    /// Transmitter power for vertical channel in kW.
    pub vertical_shv_tx_power: Real4,

    /// Calibration of system ZDR in dB.
    pub system_differential_reflectivity: Real4,

    /// Initial DP for the system in degrees.
    pub initial_system_differential_phase: Real4,

    /// Identifies the volume coverage pattern in use.
    pub volume_coverage_pattern_number: Integer2,

    /// Processing option flags.
    ///
    /// Options:
    ///   0 = RxR noise
    ///   1 = CBT
    pub processing_status: Integer2,

    /// RPG weighted mean ZDR bias estimate in dB.
    pub zdr_bias_estimate_weighted_mean: Integer2,

    /// Spare.
    pub spare: BinaryData<[u8; 6]>,
}
