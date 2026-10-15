use super::raw;
use super::{DataBlockName, DataBlockType};
use std::borrow::Cow;

/// A digital radar data block's identifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataBlockId<'a> {
    inner: Cow<'a, raw::DataBlockId>,
}

impl<'a> DataBlockId<'a> {
    /// Create a new DataBlockId wrapper from a raw DataBlockId reference.
    pub(crate) fn new(inner: &'a raw::DataBlockId) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this data block ID to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> DataBlockId<'static> {
        DataBlockId {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Data block type (raw byte), e.g. b'R'. See [`data_block_type()`](Self::data_block_type)
    /// for the decoded value.
    pub fn data_block_type_raw(&self) -> u8 {
        self.inner.data_block_type
    }

    /// Data block name (raw bytes), e.g. b"VOL". See [`data_block_name()`](Self::data_block_name)
    /// for the decoded value.
    pub fn data_block_name_raw(&self) -> &[u8; 3] {
        &self.inner.data_block_name
    }

    /// Data block type category.
    pub fn data_block_type(&self) -> DataBlockType {
        match self.inner.data_block_type {
            b'R' => DataBlockType::Radial,
            b'D' => DataBlockType::DataMoment,
            other => DataBlockType::Unknown(other),
        }
    }

    /// Data block name.
    pub fn data_block_name(&self) -> DataBlockName {
        match &self.inner.data_block_name {
            b"VOL" => DataBlockName::Volume,
            b"ELV" => DataBlockName::Elevation,
            b"RAD" => DataBlockName::Radial,
            b"REF" => DataBlockName::Reflectivity,
            b"VEL" => DataBlockName::Velocity,
            b"SW " => DataBlockName::SpectrumWidth,
            b"ZDR" => DataBlockName::DifferentialReflectivity,
            b"PHI" => DataBlockName::DifferentialPhase,
            b"RHO" => DataBlockName::CorrelationCoefficient,
            b"CFP" => DataBlockName::ClutterFilterPower,
            other => DataBlockName::Unknown(*other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name_of(data_block_name: [u8; 3]) -> DataBlockName {
        let raw = raw::DataBlockId {
            data_block_type: b'D',
            data_block_name,
        };
        DataBlockId::new(&raw).data_block_name()
    }

    #[test]
    fn spectrum_width_name_has_trailing_space() {
        assert_eq!(name_of(*b"SW "), DataBlockName::SpectrumWidth);
        assert_eq!(name_of(*b"SW\0"), DataBlockName::Unknown(*b"SW\0"));
    }
}
