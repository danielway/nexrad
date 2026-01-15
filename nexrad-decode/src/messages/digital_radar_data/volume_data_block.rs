use super::raw;
use super::{ProcessingStatus, VolumeCoveragePattern};
use crate::binary_data::BinaryData;
use std::borrow::Cow;

#[cfg(feature = "uom")]
use uom::si::f64::{Angle, Energy, Information, Length};

/// A volume data moment block.
#[derive(Clone, PartialEq, Debug)]
pub struct VolumeDataBlock<'a> {
    inner: Cow<'a, raw::VolumeDataBlock>,
}

impl<'a> VolumeDataBlock<'a> {
    /// Create a new VolumeDataBlock wrapper from a raw VolumeDataBlock reference.
    pub(crate) fn new(inner: &'a raw::VolumeDataBlock) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this volume data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> VolumeDataBlock<'static> {
        VolumeDataBlock {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Size of data block in bytes (raw value).
    pub fn lrtup_raw(&self) -> u16 {
        self.inner.lrtup.get()
    }

    /// Major version number.
    pub fn major_version_number(&self) -> u8 {
        self.inner.major_version_number
    }

    /// Minor version number.
    pub fn minor_version_number(&self) -> u8 {
        self.inner.minor_version_number
    }

    /// Latitude of radar in degrees (raw value).
    pub fn latitude_raw(&self) -> f32 {
        self.inner.latitude.get()
    }

    /// Longitude of radar in degrees (raw value).
    pub fn longitude_raw(&self) -> f32 {
        self.inner.longitude.get()
    }

    /// Height of site base above sea level in meters (raw value).
    pub fn site_height_raw(&self) -> i16 {
        self.inner.site_height.get()
    }

    /// Height of feedhorn above ground in meters (raw value).
    pub fn feedhorn_height_raw(&self) -> u16 {
        self.inner.feedhorn_height.get()
    }

    /// Reflectivity scaling factor without correction by ground noise scaling factors given in
    /// adaptation data message in dB.
    pub fn calibration_constant(&self) -> f32 {
        self.inner.calibration_constant.get()
    }

    /// Transmitter power for horizontal channel in kW (raw value).
    pub fn horizontal_shv_tx_power_raw(&self) -> f32 {
        self.inner.horizontal_shv_tx_power.get()
    }

    /// Transmitter power for vertical channel in kW (raw value).
    pub fn vertical_shv_tx_power_raw(&self) -> f32 {
        self.inner.vertical_shv_tx_power.get()
    }

    /// Calibration of system ZDR in dB.
    pub fn system_differential_reflectivity(&self) -> f32 {
        self.inner.system_differential_reflectivity.get()
    }

    /// Initial DP for the system in degrees (raw value).
    pub fn initial_system_differential_phase_raw(&self) -> f32 {
        self.inner.initial_system_differential_phase.get()
    }

    /// Identifies the volume coverage pattern in use (raw value).
    pub fn volume_coverage_pattern_number(&self) -> u16 {
        self.inner.volume_coverage_pattern_number.get()
    }

    /// Processing option flags (raw value).
    pub fn processing_status_raw(&self) -> u16 {
        self.inner.processing_status.get()
    }

    /// RPG weighted mean ZDR bias estimate in dB.
    pub fn zdr_bias_estimate_weighted_mean(&self) -> u16 {
        self.inner.zdr_bias_estimate_weighted_mean.get()
    }

    /// Spare bytes.
    pub fn spare(&self) -> &BinaryData<[u8; 6]> {
        &self.inner.spare
    }

    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.inner.lrtup.get() as f64)
    }

    /// Latitude of radar.
    #[cfg(feature = "uom")]
    pub fn latitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.inner.latitude.get() as f64)
    }

    /// Longitude of radar.
    #[cfg(feature = "uom")]
    pub fn longitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.inner.longitude.get() as f64)
    }

    /// Height of site base above sea level.
    #[cfg(feature = "uom")]
    pub fn site_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.inner.site_height.get() as f64)
    }

    /// Height of feedhorn above ground.
    #[cfg(feature = "uom")]
    pub fn feedhorn_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.inner.feedhorn_height.get() as f64)
    }

    /// Transmitter power for horizontal channel.
    #[cfg(feature = "uom")]
    pub fn horizontal_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.inner.horizontal_shv_tx_power.get() as f64)
    }

    /// Transmitter power for vertical channel.
    #[cfg(feature = "uom")]
    pub fn vertical_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.inner.vertical_shv_tx_power.get() as f64)
    }

    /// Initial DP for the system.
    #[cfg(feature = "uom")]
    pub fn initial_system_differential_phase(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(
            self.inner.initial_system_differential_phase.get() as f64
        )
    }

    /// Identifies the volume coverage pattern in use.
    pub fn volume_coverage_pattern(&self) -> VolumeCoveragePattern {
        let volume_coverage_pattern = self.inner.volume_coverage_pattern_number.get();
        match volume_coverage_pattern {
            12 => VolumeCoveragePattern::VCP12,
            31 => VolumeCoveragePattern::VCP31,
            35 => VolumeCoveragePattern::VCP35,
            112 => VolumeCoveragePattern::VCP112,
            212 => VolumeCoveragePattern::VCP212,
            215 => VolumeCoveragePattern::VCP215,
            _ => panic!(
                "Invalid volume coverage pattern number: {}",
                volume_coverage_pattern
            ),
        }
    }

    /// Processing option flags.
    pub fn processing_status(&self) -> ProcessingStatus {
        let processing_status = self.inner.processing_status.get();
        match processing_status {
            0 => ProcessingStatus::RxRNoise,
            1 => ProcessingStatus::CBT,
            _ => ProcessingStatus::Other(processing_status),
        }
    }
}
