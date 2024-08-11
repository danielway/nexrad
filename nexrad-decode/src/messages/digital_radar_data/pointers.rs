use crate::messages::primitive_aliases::Integer4;

/// A pointer to a data moment within a digital radar data message.
pub struct DataMomentPointer {
    /// The type of data moment that the pointer references.
    pub data_moment_type: DataMomentPointerType,

    /// The pointer to the data moment as a byte offset from the start of the message.
    pub pointer: Integer4,
}

/// The type of data moment that the pointer references.
#[derive(Debug)]
pub enum DataMomentPointerType {
    Volume,
    Elevation,
    Radial,
    Generic(DataMomentGenericPointerType),
}

/// The type of generic data moment that the pointer references.
#[derive(Debug)]
pub enum DataMomentGenericPointerType {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    DifferentialPhase,
    CorrelationCoefficient,
    SpecificDiffPhase,
}
