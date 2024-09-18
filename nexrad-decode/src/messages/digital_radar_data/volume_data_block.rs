use crate::messages::digital_radar_data::{DataBlockId, ProcessingStatus, VolumeCoveragePattern};
use crate::messages::primitive_aliases::{Integer1, Integer2, Real4, SInteger2};
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::si::f64::{Angle, Energy, Information, Length};

/// A volume data moment block.
#[derive(Clone, PartialEq, Deserialize)]
pub struct VolumeDataBlock {
    /// Data block identifier.
    pub data_block_id: DataBlockId,

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

    /// Height of feedhorn above ground in meters.
    pub feedhorn_height: Integer2,

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
    pub spare: [u8; 6],
}

impl VolumeDataBlock {
    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup as f64)
    }

    /// Latitude of radar.
    #[cfg(feature = "uom")]
    pub fn latitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.latitude as f64)
    }

    /// Longitude of radar.
    #[cfg(feature = "uom")]
    pub fn longitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.longitude as f64)
    }

    /// Height of site base above sea level.
    #[cfg(feature = "uom")]
    pub fn site_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.site_height as f64)
    }

    /// Height of feedhorn above ground.
    #[cfg(feature = "uom")]
    pub fn feedhorn_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.feedhorn_height as f64)
    }

    /// Transmitter power for horizontal channel.
    #[cfg(feature = "uom")]
    pub fn horizontal_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.horizontal_shv_tx_power as f64)
    }

    /// Transmitter power for vertical channel.
    #[cfg(feature = "uom")]
    pub fn vertical_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.vertical_shv_tx_power as f64)
    }

    /// Initial DP for the system.
    #[cfg(feature = "uom")]
    pub fn initial_system_differential_phase(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.initial_system_differential_phase as f64)
    }

    /// Identifies the volume coverage pattern in use.
    pub fn volume_coverage_pattern(&self) -> VolumeCoveragePattern {
        match self.volume_coverage_pattern_number {
            12 => VolumeCoveragePattern::VCP12,
            31 => VolumeCoveragePattern::VCP31,
            35 => VolumeCoveragePattern::VCP35,
            112 => VolumeCoveragePattern::VCP112,
            212 => VolumeCoveragePattern::VCP212,
            215 => VolumeCoveragePattern::VCP215,
            _ => panic!(
                "Invalid volume coverage pattern number: {}",
                self.volume_coverage_pattern_number
            ),
        }
    }

    /// Processing option flags.
    pub fn processing_status(&self) -> ProcessingStatus {
        match self.processing_status {
            0 => ProcessingStatus::RxRNoise,
            1 => ProcessingStatus::CBT,
            _ => ProcessingStatus::Other(self.processing_status),
        }
    }
}

#[cfg(not(feature = "uom"))]
impl Debug for VolumeDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolumeDataBlock")
            .field("data_block_id", &self.data_block_id)
            .field("lrtup", &self.lrtup)
            .field("major_version_number", &self.major_version_number)
            .field("minor_version_number", &self.minor_version_number)
            .field("latitude", &self.latitude)
            .field("longitude", &self.longitude)
            .field("site_height", &self.site_height)
            .field("feedhorn_height", &self.feedhorn_height)
            .field("calibration_constant", &self.calibration_constant)
            .field("horizontal_shv_tx_power", &self.horizontal_shv_tx_power)
            .field("vertical_shv_tx_power", &self.vertical_shv_tx_power)
            .field(
                "system_differential_reflectivity",
                &self.system_differential_reflectivity,
            )
            .field(
                "initial_system_differential_phase",
                &self.initial_system_differential_phase,
            )
            .field(
                "volume_coverage_pattern_number",
                &self.volume_coverage_pattern_number,
            )
            .field("processing_status", &self.processing_status())
            .field(
                "zdr_bias_estimate_weighted_mean",
                &self.zdr_bias_estimate_weighted_mean,
            )
            .field("spare", &self.spare)
            .finish()
    }
}

#[cfg(feature = "uom")]
impl Debug for VolumeDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolumeDataBlock")
            .field("data_block_id", &self.data_block_id)
            .field("lrtup", &self.lrtup())
            .field("major_version_number", &self.major_version_number)
            .field("minor_version_number", &self.minor_version_number)
            .field("latitude", &self.latitude())
            .field("longitude", &self.longitude())
            .field("site_height", &self.site_height())
            .field("feedhorn_height", &self.feedhorn_height())
            .field("calibration_constant", &self.calibration_constant)
            .field("horizontal_shv_tx_power", &self.horizontal_shv_tx_power())
            .field("vertical_shv_tx_power", &self.vertical_shv_tx_power())
            .field(
                "system_differential_reflectivity",
                &self.system_differential_reflectivity,
            )
            .field(
                "initial_system_differential_phase",
                &self.initial_system_differential_phase(),
            )
            .field(
                "volume_coverage_pattern_number",
                &self.volume_coverage_pattern_number,
            )
            .field("processing_status", &self.processing_status())
            .field(
                "zdr_bias_estimate_weighted_mean",
                &self.zdr_bias_estimate_weighted_mean,
            )
            .field("spare", &self.spare)
            .finish()
    }
}
