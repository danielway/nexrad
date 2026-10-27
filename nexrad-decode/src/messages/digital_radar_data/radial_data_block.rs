use super::raw;
use std::borrow::Cow;

/// Internal representation of the radial data block, supporting both legacy and modern formats.
///
/// The format expanded from 20 to 28 bytes at Build 12.0 (ICD 2620002K, July 2011),
/// adding channel calibration constants for dual polarization. Detection uses the
/// self-describing lrtup field.
#[derive(Clone, PartialEq, Debug)]
enum RadialDataBlockInner<'a> {
    /// Legacy format (Build 10.0–11.x, 16 bytes, lrtup = 20). No channel calibration constants.
    Legacy(Cow<'a, raw::RadialDataBlockLegacy>),
    /// Modern format (Build 12.0+, 24 bytes, lrtup = 28). Includes channel calibration constants.
    Modern(Cow<'a, raw::RadialDataBlock>),
}

/// A radial data moment block.
///
/// This type provides access to radial metadata from digital radar data messages.
/// It supports both the legacy 16-byte format (older builds) and the modern 24-byte
/// format (newer builds that include channel calibration constants).
///
/// Fields that were added in newer builds (`horizontal_channel_calibration_constant`
/// and `vertical_channel_calibration_constant`) return `Option` types that are `None`
/// for legacy data.
#[derive(Clone, PartialEq, Debug)]
pub struct RadialDataBlock<'a> {
    inner: RadialDataBlockInner<'a>,
}

impl<'a> RadialDataBlock<'a> {
    /// Create a new RadialDataBlock wrapper from a raw RadialDataBlock reference (modern format).
    pub(crate) fn new(inner: &'a raw::RadialDataBlock) -> Self {
        Self {
            inner: RadialDataBlockInner::Modern(Cow::Borrowed(inner)),
        }
    }

    /// Create a new RadialDataBlock wrapper from a raw RadialDataBlockLegacy reference.
    pub(crate) fn new_legacy(inner: &'a raw::RadialDataBlockLegacy) -> Self {
        Self {
            inner: RadialDataBlockInner::Legacy(Cow::Borrowed(inner)),
        }
    }

    /// Convert this radial data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> RadialDataBlock<'static> {
        match self.inner {
            RadialDataBlockInner::Legacy(inner) => RadialDataBlock {
                inner: RadialDataBlockInner::Legacy(Cow::Owned(inner.into_owned())),
            },
            RadialDataBlockInner::Modern(inner) => RadialDataBlock {
                inner: RadialDataBlockInner::Modern(Cow::Owned(inner.into_owned())),
            },
        }
    }

    /// Size of data block, in bytes.
    pub fn lrtup_bytes(&self) -> u16 {
        match &self.inner {
            RadialDataBlockInner::Legacy(inner) => inner.lrtup.get(),
            RadialDataBlockInner::Modern(inner) => inner.lrtup.get(),
        }
    }

    /// Unambiguous range, interval size, in tenths of a km (raw scaled value). See
    /// [`unambiguous_range_km()`](Self::unambiguous_range_km) for the decoded value.
    pub fn unambiguous_range_raw(&self) -> u16 {
        match &self.inner {
            RadialDataBlockInner::Legacy(inner) => inner.unambiguous_range.get(),
            RadialDataBlockInner::Modern(inner) => inner.unambiguous_range.get(),
        }
    }

    /// Noise level for the horizontal channel, in dBm.
    pub fn horizontal_channel_noise_level_dbm(&self) -> f32 {
        match &self.inner {
            RadialDataBlockInner::Legacy(inner) => inner.horizontal_channel_noise_level.get(),
            RadialDataBlockInner::Modern(inner) => inner.horizontal_channel_noise_level.get(),
        }
    }

    /// Noise level for the vertical channel, in dBm.
    pub fn vertical_channel_noise_level_dbm(&self) -> f32 {
        match &self.inner {
            RadialDataBlockInner::Legacy(inner) => inner.vertical_channel_noise_level.get(),
            RadialDataBlockInner::Modern(inner) => inner.vertical_channel_noise_level.get(),
        }
    }

    /// Nyquist velocity in hundredths of a m/s (raw scaled value). See
    /// [`nyquist_velocity_mps()`](Self::nyquist_velocity_mps) for the decoded value.
    pub fn nyquist_velocity_raw(&self) -> u16 {
        match &self.inner {
            RadialDataBlockInner::Legacy(inner) => inner.nyquist_velocity.get(),
            RadialDataBlockInner::Modern(inner) => inner.nyquist_velocity.get(),
        }
    }

    /// Radial flags to support RPG processing (raw bitfield).
    pub fn radial_flags_raw(&self) -> u16 {
        match &self.inner {
            RadialDataBlockInner::Legacy(inner) => inner.radial_flags.get(),
            RadialDataBlockInner::Modern(inner) => inner.radial_flags.get(),
        }
    }

    /// Calibration constant for the horizontal channel in dBZ.
    ///
    /// Returns `None` for legacy data (older builds) as this field was not
    /// present in earlier ICD revisions.
    pub fn horizontal_channel_calibration_constant_db(&self) -> Option<f32> {
        match &self.inner {
            RadialDataBlockInner::Legacy(_) => None,
            RadialDataBlockInner::Modern(inner) => {
                Some(inner.horizontal_channel_calibration_constant.get())
            }
        }
    }

    /// Calibration constant for the vertical channel in dBZ.
    ///
    /// Returns `None` for legacy data (older builds) as this field was not
    /// present in earlier ICD revisions.
    pub fn vertical_channel_calibration_constant_db(&self) -> Option<f32> {
        match &self.inner {
            RadialDataBlockInner::Legacy(_) => None,
            RadialDataBlockInner::Modern(inner) => {
                Some(inner.vertical_channel_calibration_constant.get())
            }
        }
    }

    /// Unambiguous range, interval size, in km.
    pub fn unambiguous_range_km(&self) -> f32 {
        self.unambiguous_range_raw() as f32 * 0.1
    }

    /// Nyquist velocity, in m/s.
    pub fn nyquist_velocity_mps(&self) -> f32 {
        self.nyquist_velocity_raw() as f32 * 0.01
    }
}
