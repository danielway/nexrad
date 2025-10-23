use crate::binary_data::BinaryData;
use crate::messages::digital_radar_data::{DataBlockId, ProcessingStatus, VolumeCoveragePattern};
use crate::messages::primitive_aliases::{Integer1, Integer2, Real4, SInteger2};
use std::fmt::Debug;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[cfg(feature = "uom")]
use uom::si::f64::{Angle, Energy, Information, Length};

/// A volume data moment block.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, TryFromBytes, Immutable, KnownLayout)]
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
    pub spare: BinaryData<[u8; 6]>,
}

impl VolumeDataBlock {
    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup.get() as f64)
    }

    /// Latitude of radar.
    #[cfg(feature = "uom")]
    pub fn latitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.latitude.get() as f64)
    }

    /// Longitude of radar.
    #[cfg(feature = "uom")]
    pub fn longitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.longitude.get() as f64)
    }

    /// Height of site base above sea level.
    #[cfg(feature = "uom")]
    pub fn site_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.site_height.get() as f64)
    }

    /// Height of feedhorn above ground.
    #[cfg(feature = "uom")]
    pub fn feedhorn_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.feedhorn_height.get() as f64)
    }

    /// Transmitter power for horizontal channel.
    #[cfg(feature = "uom")]
    pub fn horizontal_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.horizontal_shv_tx_power.get() as f64)
    }

    /// Transmitter power for vertical channel.
    #[cfg(feature = "uom")]
    pub fn vertical_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.vertical_shv_tx_power.get() as f64)
    }

    /// Initial DP for the system.
    #[cfg(feature = "uom")]
    pub fn initial_system_differential_phase(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.initial_system_differential_phase.get() as f64)
    }

    /// Identifies the volume coverage pattern in use.
    pub fn volume_coverage_pattern(&self) -> VolumeCoveragePattern {
        match self.volume_coverage_pattern_number.get() {
            12 => VolumeCoveragePattern::VCP12,
            31 => VolumeCoveragePattern::VCP31,
            35 => VolumeCoveragePattern::VCP35,
            112 => VolumeCoveragePattern::VCP112,
            212 => VolumeCoveragePattern::VCP212,
            215 => VolumeCoveragePattern::VCP215,
            _ => panic!(
                "Invalid volume coverage pattern number: {}",
                self.volume_coverage_pattern_number.get()
            ),
        }
    }

    /// Processing option flags.
    pub fn processing_status(&self) -> ProcessingStatus {
        match self.processing_status.get() {
            0 => ProcessingStatus::RxRNoise,
            1 => ProcessingStatus::CBT,
            _ => ProcessingStatus::Other(self.processing_status.get()),
        }
    }

    /// Decodes a reference to a VolumeDataBlock from a byte slice, returning the block and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a VolumeDataBlock from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (block, _) = Self::decode_ref(bytes)?;
        Ok(block.clone())
    }
}
