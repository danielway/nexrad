#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Identifies a radar data product (moment type).
///
/// Each product corresponds to a different type of moment data captured by the radar.
/// This enum is shared across the entire crate ecosystem — it identifies which moment
/// field to extract from a radial, which processing algorithm to apply, or which color
/// scale to use for rendering.
///
/// The enum is non-exhaustive so that a moment added by a later ICD build can be named in a
/// minor release. [`Product::ALL`] lists every variant this version knows, and
/// [`Product::as_str`] and [`FromStr`] convert to and from a stable name.
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
    /// Every product this version of the crate knows, in declaration order.
    ///
    /// The enum is non-exhaustive, so a later minor release may append to this list.
    pub const ALL: &'static [Product] = &[
        Product::Reflectivity,
        Product::Velocity,
        Product::SpectrumWidth,
        Product::DifferentialReflectivity,
        Product::DifferentialPhase,
        Product::CorrelationCoefficient,
        Product::ClutterFilterPower,
    ];

    /// Returns a stable snake_case name for this product, such as `"reflectivity"` or
    /// `"spectrum_width"`.
    ///
    /// The name does not change between releases, so it is suitable for configuration files,
    /// URLs and storage keys. [`FromStr`] parses it back, and [`Display`] writes it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Product::Reflectivity => "reflectivity",
            Product::Velocity => "velocity",
            Product::SpectrumWidth => "spectrum_width",
            Product::DifferentialReflectivity => "differential_reflectivity",
            Product::DifferentialPhase => "differential_phase",
            Product::CorrelationCoefficient => "correlation_coefficient",
            Product::ClutterFilterPower => "clutter_filter_power",
        }
    }

    /// Returns the three-character data block name the ICD uses for this product's moment in a
    /// digital radar data message: `"REF"`, `"VEL"`, `"SW"`, `"ZDR"`, `"PHI"`, `"RHO"` or `"CFP"`.
    ///
    /// On the wire the spectrum width block name is `"SW "`, padded with a space; this method
    /// returns it without the padding.
    pub fn data_block_name(&self) -> &'static str {
        match self {
            Product::Reflectivity => "REF",
            Product::Velocity => "VEL",
            Product::SpectrumWidth => "SW",
            Product::DifferentialReflectivity => "ZDR",
            Product::DifferentialPhase => "PHI",
            Product::CorrelationCoefficient => "RHO",
            Product::ClutterFilterPower => "CFP",
        }
    }

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

impl Display for Product {
    /// Writes the stable name returned by [`Product::as_str`].
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Product {
    type Err = crate::result::Error;

    /// Parses a product from either of two spellings:
    ///
    /// - the stable name returned by [`Product::as_str`], such as `"spectrum_width"`, matched
    ///   exactly;
    /// - the ICD data block name returned by [`Product::data_block_name`] (`"REF"`, `"VEL"`,
    ///   `"SW"`, `"ZDR"`, `"PHI"`, `"RHO"`, `"CFP"`), matched ignoring ASCII case and surrounding
    ///   whitespace, so the padded wire name `"SW "` also parses.
    ///
    /// Any other input returns [`Error::UnknownProduct`](crate::result::Error::UnknownProduct).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(product) = Product::ALL.iter().find(|p| p.as_str() == s) {
            return Ok(*product);
        }
        let trimmed = s.trim();
        Product::ALL
            .iter()
            .find(|p| p.data_block_name().eq_ignore_ascii_case(trimmed))
            .copied()
            .ok_or_else(|| crate::result::Error::UnknownProduct {
                name: s.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lists_each_variant_once() {
        for (i, a) in Product::ALL.iter().enumerate() {
            for b in &Product::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(Product::ALL.len(), 7);
    }

    #[test]
    fn names_round_trip() {
        for product in Product::ALL {
            assert_eq!(product.as_str().parse::<Product>(), Ok(*product));
            assert_eq!(product.to_string().parse::<Product>(), Ok(*product));
            assert_eq!(product.data_block_name().parse::<Product>(), Ok(*product));
            assert_eq!(
                product
                    .data_block_name()
                    .to_ascii_lowercase()
                    .parse::<Product>(),
                Ok(*product)
            );
        }
    }

    #[test]
    fn parses_padded_wire_block_name() {
        assert_eq!("SW ".parse::<Product>(), Ok(Product::SpectrumWidth));
    }

    #[test]
    fn rejects_unknown_names() {
        for name in ["", "Reflectivity", "REFL", "DREF", "spectrum width"] {
            assert_eq!(
                name.parse::<Product>(),
                Err(crate::result::Error::UnknownProduct {
                    name: name.to_string()
                })
            );
        }
    }
}
