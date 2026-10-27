use std::fmt::{Display, Formatter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Pulse width of a volume coverage pattern, as the RDA/RPG ICD codes it.
///
/// This type has the same variants as `nexrad_model::data::PulseWidth`, and the `From` impls
/// between them (feature `model`) convert every value, including [`Unknown`](Self::Unknown) with
/// its code. This copy also knows the ICD code for each variant: see
/// [`from_number`](Self::from_number) and [`number`](Self::number).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PulseWidth {
    /// Short pulse (code 2).
    Short,
    /// Long pulse (code 4).
    Long,
    /// A pulse width code the ICD does not define, carrying the code.
    Unknown(u8),
}

impl PulseWidth {
    /// Classifies a pulse width code as it appears on the wire. Codes the ICD does not define map
    /// to [`PulseWidth::Unknown`].
    pub fn from_number(code: u8) -> Self {
        match code {
            2 => PulseWidth::Short,
            4 => PulseWidth::Long,
            other => PulseWidth::Unknown(other),
        }
    }

    /// The pulse width code as it appears on the wire.
    pub fn number(self) -> u8 {
        match self {
            PulseWidth::Short => 2,
            PulseWidth::Long => 4,
            PulseWidth::Unknown(code) => code,
        }
    }
}

impl Display for PulseWidth {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PulseWidth::Short => write!(f, "Short"),
            PulseWidth::Long => write!(f, "Long"),
            PulseWidth::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}

#[cfg(feature = "model")]
impl From<PulseWidth> for nexrad_model::data::PulseWidth {
    fn from(value: PulseWidth) -> Self {
        match value {
            PulseWidth::Short => Self::Short,
            PulseWidth::Long => Self::Long,
            PulseWidth::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(feature = "model")]
impl From<nexrad_model::data::PulseWidth> for PulseWidth {
    fn from(value: nexrad_model::data::PulseWidth) -> Self {
        use nexrad_model::data::PulseWidth as Model;

        match value {
            Model::Short => Self::Short,
            Model::Long => Self::Long,
            Model::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PulseWidth;

    #[test]
    fn number_round_trips_for_every_code() {
        for code in 0..=u8::MAX {
            assert_eq!(PulseWidth::from_number(code).number(), code);
        }
        assert_eq!(PulseWidth::from_number(4), PulseWidth::Long);
        assert_eq!(PulseWidth::from_number(1), PulseWidth::Unknown(1));
    }

    #[test]
    fn displays_name() {
        assert_eq!(PulseWidth::Short.to_string(), "Short");
        assert_eq!(PulseWidth::Unknown(1).to_string(), "Unknown (1)");
    }

    #[cfg(feature = "model")]
    #[test]
    fn converts_to_and_from_the_model_without_loss() {
        for code in 0..=u8::MAX {
            let decoded = PulseWidth::from_number(code);
            let model = nexrad_model::data::PulseWidth::from(decoded);
            assert_eq!(PulseWidth::from(model), decoded);
        }
    }
}
