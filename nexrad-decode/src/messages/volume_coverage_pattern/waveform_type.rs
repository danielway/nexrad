use std::fmt::{Display, Formatter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Waveform type of an elevation cut, as the RDA/RPG ICD codes it.
///
/// This type has the same variants as `nexrad_model::data::WaveformType`, and the `From` impls
/// between them (feature `model`) convert every value, including [`Unknown`](Self::Unknown) with
/// its code. This copy also knows the ICD code for each variant: see
/// [`from_number`](Self::from_number) and [`number`](Self::number).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WaveformType {
    /// Contiguous Surveillance (code 1).
    ContiguousSurveillance,
    /// Contiguous Doppler with Ambiguity Resolution (code 2).
    ContiguousDopplerWithAmbiguityResolution,
    /// Contiguous Doppler without Ambiguity Resolution (code 3).
    ContiguousDopplerWithoutAmbiguityResolution,
    /// Batch (code 4).
    Batch,
    /// Staggered Pulse Pair (code 5).
    StaggeredPulsePair,
    /// A waveform type code the ICD does not define, carrying the code.
    Unknown(u8),
}

impl WaveformType {
    /// Classifies a waveform type code as it appears on the wire. Codes the ICD does not define
    /// map to [`WaveformType::Unknown`].
    pub fn from_number(code: u8) -> Self {
        match code {
            1 => WaveformType::ContiguousSurveillance,
            2 => WaveformType::ContiguousDopplerWithAmbiguityResolution,
            3 => WaveformType::ContiguousDopplerWithoutAmbiguityResolution,
            4 => WaveformType::Batch,
            5 => WaveformType::StaggeredPulsePair,
            other => WaveformType::Unknown(other),
        }
    }

    /// The waveform type code as it appears on the wire.
    pub fn number(self) -> u8 {
        match self {
            WaveformType::ContiguousSurveillance => 1,
            WaveformType::ContiguousDopplerWithAmbiguityResolution => 2,
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => 3,
            WaveformType::Batch => 4,
            WaveformType::StaggeredPulsePair => 5,
            WaveformType::Unknown(code) => code,
        }
    }

    /// The abbreviation the RDA/RPG ICD's volume coverage pattern tables use for this waveform
    /// type: `CS`, `CD/W`, `CD/WO` or `B`.
    ///
    /// Returns `None` for [`StaggeredPulsePair`](Self::StaggeredPulsePair), which the ICD's
    /// waveform type legend does not abbreviate, and for [`Unknown`](Self::Unknown).
    pub fn abbreviation(self) -> Option<&'static str> {
        match self {
            WaveformType::ContiguousSurveillance => Some("CS"),
            WaveformType::ContiguousDopplerWithAmbiguityResolution => Some("CD/W"),
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => Some("CD/WO"),
            WaveformType::Batch => Some("B"),
            WaveformType::StaggeredPulsePair | WaveformType::Unknown(_) => None,
        }
    }
}

impl Display for WaveformType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveformType::ContiguousSurveillance => write!(f, "Contiguous Surveillance"),
            WaveformType::ContiguousDopplerWithAmbiguityResolution => {
                write!(f, "Contiguous Doppler w/ Ambiguity Resolution")
            }
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => {
                write!(f, "Contiguous Doppler w/o Ambiguity Resolution")
            }
            WaveformType::Batch => write!(f, "Batch"),
            WaveformType::StaggeredPulsePair => write!(f, "Staggered Pulse Pair"),
            WaveformType::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}

#[cfg(feature = "model")]
impl From<WaveformType> for nexrad_model::data::WaveformType {
    fn from(value: WaveformType) -> Self {
        match value {
            WaveformType::ContiguousSurveillance => Self::ContiguousSurveillance,
            WaveformType::ContiguousDopplerWithAmbiguityResolution => {
                Self::ContiguousDopplerWithAmbiguityResolution
            }
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => {
                Self::ContiguousDopplerWithoutAmbiguityResolution
            }
            WaveformType::Batch => Self::Batch,
            WaveformType::StaggeredPulsePair => Self::StaggeredPulsePair,
            WaveformType::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(feature = "model")]
impl From<nexrad_model::data::WaveformType> for WaveformType {
    fn from(value: nexrad_model::data::WaveformType) -> Self {
        use nexrad_model::data::WaveformType as Model;

        match value {
            Model::ContiguousSurveillance => Self::ContiguousSurveillance,
            Model::ContiguousDopplerWithAmbiguityResolution => {
                Self::ContiguousDopplerWithAmbiguityResolution
            }
            Model::ContiguousDopplerWithoutAmbiguityResolution => {
                Self::ContiguousDopplerWithoutAmbiguityResolution
            }
            Model::Batch => Self::Batch,
            Model::StaggeredPulsePair => Self::StaggeredPulsePair,
            Model::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WaveformType;

    #[test]
    fn number_round_trips_for_every_code() {
        for code in 0..=u8::MAX {
            assert_eq!(WaveformType::from_number(code).number(), code);
        }
        assert_eq!(WaveformType::from_number(0), WaveformType::Unknown(0));
        assert_eq!(WaveformType::from_number(4), WaveformType::Batch);
    }

    #[test]
    fn abbreviations_follow_the_icd_legend() {
        assert_eq!(
            WaveformType::ContiguousSurveillance.abbreviation(),
            Some("CS")
        );
        assert_eq!(
            WaveformType::ContiguousDopplerWithAmbiguityResolution.abbreviation(),
            Some("CD/W")
        );
        assert_eq!(
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution.abbreviation(),
            Some("CD/WO")
        );
        assert_eq!(WaveformType::Batch.abbreviation(), Some("B"));
        assert_eq!(WaveformType::StaggeredPulsePair.abbreviation(), None);
        assert_eq!(WaveformType::Unknown(9).abbreviation(), None);
    }

    #[test]
    fn displays_long_form() {
        assert_eq!(
            WaveformType::ContiguousSurveillance.to_string(),
            "Contiguous Surveillance"
        );
        assert_eq!(WaveformType::Unknown(9).to_string(), "Unknown (9)");
    }

    #[cfg(feature = "model")]
    #[test]
    fn converts_to_and_from_the_model_without_loss() {
        for code in 0..=u8::MAX {
            let decoded = WaveformType::from_number(code);
            let model = nexrad_model::data::WaveformType::from(decoded);
            assert_eq!(WaveformType::from(model), decoded);
        }
    }
}
