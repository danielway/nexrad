#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Identifies a radar data product (moment type).
///
/// Each product corresponds to a different type of moment data captured by the radar.
/// This enum is shared across the entire crate ecosystem — it identifies which moment
/// field to extract from a radial, which processing algorithm to apply, or which color
/// scale to use for rendering.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum Product {
    /// Reflectivity (dBZ): returned power expressed in dBZ.
    Reflectivity,
    /// Radial velocity (m/s): the component of target velocity along the beam, toward or
    /// away from the radar.
    Velocity,
    /// Spectrum width (m/s): the spread of radial velocities within the sample volume.
    SpectrumWidth,
    /// Differential reflectivity (dB): the ratio of the horizontally to the vertically
    /// polarized reflectivity, in dB.
    DifferentialReflectivity,
    /// Differential phase (degrees): the phase difference between the horizontally and
    /// vertically polarized returns.
    DifferentialPhase,
    /// Correlation coefficient (unitless): the correlation between the horizontally and
    /// vertically polarized returns.
    CorrelationCoefficient,
    /// Clutter filter power (dB): the difference between clutter-filtered and unfiltered
    /// reflectivity.
    ClutterFilterPower,
}

impl Product {
    /// Returns the display range `(min, max)` for this product.
    ///
    /// This is a display default: the range a renderer normalizes the product's values to
    /// when no other range is given (`nexrad-render` builds its color lookup table over it).
    /// It is not the physical or encodable range of the moment, and decoded values can fall
    /// outside it (for example correlation coefficients above 1.0); `nexrad-render` clamps
    /// them to the ends of the range. The ranges are stable for 1.x.
    pub fn value_range(&self) -> (f32, f32) {
        match self {
            Product::Reflectivity => (-32.0, 95.0),
            Product::Velocity => (-64.0, 64.0),
            Product::SpectrumWidth => (0.0, 30.0),
            Product::DifferentialReflectivity => (-2.0, 6.0),
            Product::DifferentialPhase => (0.0, 360.0),
            Product::CorrelationCoefficient => (0.0, 1.0),
            Product::ClutterFilterPower => (-20.0, 20.0),
        }
    }

    /// Returns the standard unit string for this product.
    pub fn unit(&self) -> &'static str {
        match self {
            Product::Reflectivity => "dBZ",
            Product::Velocity => "m/s",
            Product::SpectrumWidth => "m/s",
            Product::DifferentialReflectivity => "dB",
            Product::DifferentialPhase => "°",
            Product::CorrelationCoefficient => "",
            Product::ClutterFilterPower => "dB",
        }
    }

    /// Returns a human-readable label for this product.
    pub fn label(&self) -> &'static str {
        match self {
            Product::Reflectivity => "Reflectivity",
            Product::Velocity => "Velocity",
            Product::SpectrumWidth => "Spectrum Width",
            Product::DifferentialReflectivity => "Differential Reflectivity",
            Product::DifferentialPhase => "Differential Phase",
            Product::CorrelationCoefficient => "Correlation Coefficient",
            Product::ClutterFilterPower => "Clutter Filter Power",
        }
    }
}
