use super::raw;
use std::borrow::Cow;

/// An elevation data block.
#[derive(Clone, PartialEq, Debug)]
pub struct ElevationDataBlock<'a> {
    inner: Cow<'a, raw::ElevationDataBlock>,
}

impl<'a> ElevationDataBlock<'a> {
    /// Create a new ElevationDataBlock wrapper from a raw ElevationDataBlock reference.
    pub(crate) fn new(inner: &'a raw::ElevationDataBlock) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this elevation data block to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> ElevationDataBlock<'static> {
        ElevationDataBlock {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Size of data block, in bytes.
    pub fn lrtup_bytes(&self) -> u16 {
        self.inner.lrtup.get()
    }

    /// Atmospheric attenuation factor in thousandths of a dB/km (raw scaled value). See
    /// [`atmospheric_attenuation_db_per_km()`](Self::atmospheric_attenuation_db_per_km) for the
    /// decoded value.
    pub fn atmospheric_attenuation_raw(&self) -> i16 {
        self.inner.atmospheric_attenuation.get()
    }

    /// Scaling constant used by the signal processor for this elevation to calculate reflectivity
    /// in dB.
    pub fn calibration_constant_db(&self) -> f32 {
        self.inner.calibration_constant.get()
    }

    /// Atmospheric attenuation factor, in dB/km.
    pub fn atmospheric_attenuation_db_per_km(&self) -> f32 {
        self.inner.atmospheric_attenuation.get() as f32 / 1000.0
    }
}
