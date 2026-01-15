use super::raw;
use std::borrow::Cow;

#[cfg(feature = "uom")]
use uom::si::f64::{Information, Length, Velocity};

/// A radial data moment block.
#[derive(Clone, PartialEq, Debug)]
pub struct RadialDataBlock<'a> {
    inner: Cow<'a, raw::RadialDataBlock>,
}

impl<'a> RadialDataBlock<'a> {
    /// Create a new RadialDataBlock wrapper from a raw RadialDataBlock reference.
    pub(crate) fn new(inner: &'a raw::RadialDataBlock) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this radial data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> RadialDataBlock<'static> {
        RadialDataBlock {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Size of data block in bytes (raw value).
    pub fn lrtup_raw(&self) -> u16 {
        self.inner.lrtup.get()
    }

    /// Unambiguous range, interval size, in km (raw scaled value).
    pub fn unambiguous_range_raw(&self) -> u16 {
        self.inner.unambiguous_range.get()
    }

    /// Noise level for the horizontal channel in dBm.
    pub fn horizontal_channel_noise_level(&self) -> f32 {
        self.inner.horizontal_channel_noise_level.get()
    }

    /// Noise level for the vertical channel in dBm.
    pub fn vertical_channel_noise_level(&self) -> f32 {
        self.inner.vertical_channel_noise_level.get()
    }

    /// Nyquist velocity in m/s (raw scaled value).
    pub fn nyquist_velocity_raw(&self) -> u16 {
        self.inner.nyquist_velocity.get()
    }

    /// Radial flags to support RPG processing.
    pub fn radial_flags(&self) -> u16 {
        self.inner.radial_flags.get()
    }

    /// Calibration constant for the horizontal channel in dBZ.
    pub fn horizontal_channel_calibration_constant(&self) -> f32 {
        self.inner.horizontal_channel_calibration_constant.get()
    }

    /// Calibration constant for the vertical channel in dBZ.
    pub fn vertical_channel_calibration_constant(&self) -> f32 {
        self.inner.vertical_channel_calibration_constant.get()
    }

    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.inner.lrtup.get() as f64)
    }

    /// Unambiguous range, interval size.
    #[cfg(feature = "uom")]
    pub fn unambiguous_range(&self) -> Length {
        Length::new::<uom::si::length::kilometer>(self.inner.unambiguous_range.get() as f64)
    }

    /// Nyquist velocity.
    #[cfg(feature = "uom")]
    pub fn nyquist_velocity(&self) -> Velocity {
        Velocity::new::<uom::si::velocity::meter_per_second>(
            self.inner.nyquist_velocity.get() as f64 * 0.01,
        )
    }
}
