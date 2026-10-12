/// The named identity of a data block.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum DataBlockName {
    /// Volume metadata block ("VOL").
    Volume,
    /// Elevation metadata block ("ELV").
    Elevation,
    /// Radial metadata block ("RAD").
    Radial,
    /// Reflectivity data moment ("REF").
    Reflectivity,
    /// Velocity data moment ("VEL").
    Velocity,
    /// Spectrum width data moment ("SW ", with a trailing space).
    SpectrumWidth,
    /// Differential reflectivity data moment ("ZDR").
    DifferentialReflectivity,
    /// Differential phase data moment ("PHI").
    DifferentialPhase,
    /// Correlation coefficient data moment ("RHO").
    CorrelationCoefficient,
    /// Clutter filter power data moment ("CFP").
    ClutterFilterPower,
    /// Unknown data block name for forward compatibility.
    Unknown([u8; 3]),
}

impl DataBlockName {
    /// The model product carried by a data moment block with this name, or `None` for the
    /// metadata blocks ([`Volume`](Self::Volume), [`Elevation`](Self::Elevation),
    /// [`Radial`](Self::Radial)) and unknown names.
    #[cfg(feature = "model")]
    pub fn product(&self) -> Option<nexrad_model::data::Product> {
        use nexrad_model::data::Product;

        match self {
            DataBlockName::Reflectivity => Some(Product::Reflectivity),
            DataBlockName::Velocity => Some(Product::Velocity),
            DataBlockName::SpectrumWidth => Some(Product::SpectrumWidth),
            DataBlockName::DifferentialReflectivity => Some(Product::DifferentialReflectivity),
            DataBlockName::DifferentialPhase => Some(Product::DifferentialPhase),
            DataBlockName::CorrelationCoefficient => Some(Product::CorrelationCoefficient),
            DataBlockName::ClutterFilterPower => Some(Product::ClutterFilterPower),
            DataBlockName::Volume
            | DataBlockName::Elevation
            | DataBlockName::Radial
            | DataBlockName::Unknown(_) => None,
        }
    }
}
