use crate::messages::primitive_aliases::Code2;
use std::fmt::Debug;

/// The types of data that have transmission enabled.
pub struct DataTransmissionEnabled(Code2);

impl DataTransmissionEnabled {
    pub(crate) fn new(value: Code2) -> Self {
        Self(value)
    }

    /// Whether no data types have transmission enabled.
    pub fn none(&self) -> bool {
        self.0 & 0b0001 != 0
    }

    /// Whether reflectivity data has transmission enabled.
    pub fn reflectivity(&self) -> bool {
        self.0 & 0b0010 != 0
    }

    /// Whether velocity data has transmission enabled.
    pub fn velocity(&self) -> bool {
        self.0 & 0b0100 != 0
    }

    /// Whether spectrum width data has transmission enabled.
    pub fn spectrum_width(&self) -> bool {
        self.0 & 0b1000 != 0
    }
}

impl Debug for DataTransmissionEnabled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTransmissionEnabled")
            .field("none", &self.none())
            .field("reflectivity", &self.reflectivity())
            .field("velocity", &self.velocity())
            .field("spectrum_width", &self.spectrum_width())
            .finish()
    }
}
