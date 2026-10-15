use super::raw;
use super::ProcessingStatus;
use crate::messages::VcpNumber;
use std::borrow::Cow;

#[cfg(feature = "uom")]
use uom::si::f64::{Angle, Information, Length, Power};

/// Internal representation of the volume data block, supporting both legacy and modern formats.
///
/// The format expanded at Build 20.0 (ICD 2620002U, July 2021).
#[derive(Clone, PartialEq, Debug)]
enum VolumeDataBlockInner<'a> {
    /// Legacy format (Build 10.0–19.0, 40 bytes, lrtup = 44).
    Legacy(Cow<'a, raw::VolumeDataBlockLegacy>),
    /// Modern format (Build 20.0+, 48 bytes, lrtup = 52).
    Modern(Cow<'a, raw::VolumeDataBlock>),
}

/// A volume data moment block.
///
/// This type provides access to volume metadata from digital radar data messages.
/// It supports both the legacy 40-byte format (Build 19.0 and earlier) and the
/// modern 48-byte format (Build 20.0 and later).
///
/// Fields that were added in Build 20.0 (`zdr_bias_estimate_weighted_mean` and `spare`)
/// return `Option` types that are `None` for legacy data.
#[derive(Clone, PartialEq, Debug)]
pub struct VolumeDataBlock<'a> {
    inner: VolumeDataBlockInner<'a>,
}

impl<'a> VolumeDataBlock<'a> {
    /// Create a new VolumeDataBlock wrapper from a raw VolumeDataBlock reference (modern format).
    pub(crate) fn new(inner: &'a raw::VolumeDataBlock) -> Self {
        Self {
            inner: VolumeDataBlockInner::Modern(Cow::Borrowed(inner)),
        }
    }

    /// Create a new VolumeDataBlock wrapper from a raw VolumeDataBlockLegacy reference.
    pub(crate) fn new_legacy(inner: &'a raw::VolumeDataBlockLegacy) -> Self {
        Self {
            inner: VolumeDataBlockInner::Legacy(Cow::Borrowed(inner)),
        }
    }

    /// Convert this volume data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> VolumeDataBlock<'static> {
        match self.inner {
            VolumeDataBlockInner::Legacy(inner) => VolumeDataBlock {
                inner: VolumeDataBlockInner::Legacy(Cow::Owned(inner.into_owned())),
            },
            VolumeDataBlockInner::Modern(inner) => VolumeDataBlock {
                inner: VolumeDataBlockInner::Modern(Cow::Owned(inner.into_owned())),
            },
        }
    }

    /// Size of data block, in bytes.
    pub fn lrtup_bytes(&self) -> u16 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.lrtup.get(),
            VolumeDataBlockInner::Modern(inner) => inner.lrtup.get(),
        }
    }

    /// Major version number.
    pub fn major_version_number(&self) -> u8 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.major_version_number,
            VolumeDataBlockInner::Modern(inner) => inner.major_version_number,
        }
    }

    /// Minor version number.
    pub fn minor_version_number(&self) -> u8 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.minor_version_number,
            VolumeDataBlockInner::Modern(inner) => inner.minor_version_number,
        }
    }

    /// Latitude of radar, in degrees.
    pub fn latitude_degrees(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.latitude.get(),
            VolumeDataBlockInner::Modern(inner) => inner.latitude.get(),
        }
    }

    /// Longitude of radar, in degrees.
    pub fn longitude_degrees(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.longitude.get(),
            VolumeDataBlockInner::Modern(inner) => inner.longitude.get(),
        }
    }

    /// Height of site base above sea level, in meters.
    pub fn site_height_meters(&self) -> i16 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.site_height.get(),
            VolumeDataBlockInner::Modern(inner) => inner.site_height.get(),
        }
    }

    /// Height of radar tower above ground, in meters.
    pub fn tower_height_meters(&self) -> u16 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.tower_height.get(),
            VolumeDataBlockInner::Modern(inner) => inner.tower_height.get(),
        }
    }

    /// Reflectivity scaling factor without correction by ground noise scaling factors given in
    /// adaptation data message in dB.
    pub fn calibration_constant_db(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.calibration_constant.get(),
            VolumeDataBlockInner::Modern(inner) => inner.calibration_constant.get(),
        }
    }

    /// Transmitter power for horizontal channel, in kW.
    pub fn horizontal_shv_tx_power_kw(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.horizontal_shv_tx_power.get(),
            VolumeDataBlockInner::Modern(inner) => inner.horizontal_shv_tx_power.get(),
        }
    }

    /// Transmitter power for vertical channel, in kW.
    pub fn vertical_shv_tx_power_kw(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.vertical_shv_tx_power.get(),
            VolumeDataBlockInner::Modern(inner) => inner.vertical_shv_tx_power.get(),
        }
    }

    /// Calibration of system ZDR in dB.
    pub fn system_differential_reflectivity_db(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.system_differential_reflectivity.get(),
            VolumeDataBlockInner::Modern(inner) => inner.system_differential_reflectivity.get(),
        }
    }

    /// Initial DP for the system, in degrees.
    pub fn initial_system_differential_phase_degrees(&self) -> f32 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.initial_system_differential_phase.get(),
            VolumeDataBlockInner::Modern(inner) => inner.initial_system_differential_phase.get(),
        }
    }

    /// Identifies the volume coverage pattern in use (raw value). See
    /// [`vcp_number()`](Self::vcp_number) for the decoded value.
    pub fn vcp_number_raw(&self) -> u16 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.volume_coverage_pattern_number.get(),
            VolumeDataBlockInner::Modern(inner) => inner.volume_coverage_pattern_number.get(),
        }
    }

    /// Processing option flags (raw value).
    pub fn processing_status_raw(&self) -> u16 {
        match &self.inner {
            VolumeDataBlockInner::Legacy(inner) => inner.processing_status.get(),
            VolumeDataBlockInner::Modern(inner) => inner.processing_status.get(),
        }
    }

    /// RPG weighted mean ZDR bias estimate (raw coded value, encoded like the ZDR data moment
    /// with 0 meaning not available). See
    /// [`zdr_bias_estimate_weighted_mean_db()`](Self::zdr_bias_estimate_weighted_mean_db) for the
    /// decoded value.
    ///
    /// Returns `None` for legacy data (Build 19.0 and earlier) as this field
    /// was added in Build 20.0.
    pub fn zdr_bias_estimate_weighted_mean_raw(&self) -> Option<u16> {
        match &self.inner {
            VolumeDataBlockInner::Legacy(_) => None,
            VolumeDataBlockInner::Modern(inner) => {
                Some(inner.zdr_bias_estimate_weighted_mean.get())
            }
        }
    }

    /// RPG weighted mean ZDR bias estimate in dB, decoded as `(raw - 418) / 32`.
    ///
    /// Returns `None` for legacy data (Build 19.0 and earlier), which lacks this field, and when
    /// the coded value is 0 (not available).
    pub fn zdr_bias_estimate_weighted_mean_db(&self) -> Option<f32> {
        match self.zdr_bias_estimate_weighted_mean_raw()? {
            0 => None,
            raw => Some((raw as f32 - 418.0) / 32.0),
        }
    }

    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup_bytes() as f64)
    }

    /// Latitude of radar.
    #[cfg(feature = "uom")]
    pub fn latitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.latitude_degrees() as f64)
    }

    /// Longitude of radar.
    #[cfg(feature = "uom")]
    pub fn longitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.longitude_degrees() as f64)
    }

    /// Height of site base above sea level.
    #[cfg(feature = "uom")]
    pub fn site_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.site_height_meters() as f64)
    }

    /// Height of radar tower above ground.
    #[cfg(feature = "uom")]
    pub fn tower_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.tower_height_meters() as f64)
    }

    /// Transmitter power for horizontal channel.
    #[cfg(feature = "uom")]
    pub fn horizontal_shv_tx_power(&self) -> Power {
        Power::new::<uom::si::power::kilowatt>(self.horizontal_shv_tx_power_kw() as f64)
    }

    /// Transmitter power for vertical channel.
    #[cfg(feature = "uom")]
    pub fn vertical_shv_tx_power(&self) -> Power {
        Power::new::<uom::si::power::kilowatt>(self.vertical_shv_tx_power_kw() as f64)
    }

    /// Initial DP for the system.
    #[cfg(feature = "uom")]
    pub fn initial_system_differential_phase(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.initial_system_differential_phase_degrees() as f64)
    }

    /// Identifies the volume coverage pattern in use.
    pub fn vcp_number(&self) -> VcpNumber {
        VcpNumber::from_number(self.vcp_number_raw())
    }

    /// Processing option flags.
    pub fn processing_status(&self) -> ProcessingStatus {
        let processing_status = self.processing_status_raw();
        match processing_status {
            0 => ProcessingStatus::RxrNoise,
            1 => ProcessingStatus::Cbt,
            _ => ProcessingStatus::Other(processing_status),
        }
    }
}
