use std::fmt::{Display, Formatter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A radial's position within its elevation sweep and volume scan, as the RDA/RPG ICD codes it.
///
/// This type has the same variants as `nexrad_model::data::RadialStatus`, and the `From` impls
/// between them (feature `model`) convert every value, including [`Unknown`](Self::Unknown) with
/// its code. This copy also knows the ICD code for each variant: see
/// [`from_number`](Self::from_number) and [`number`](Self::number).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RadialStatus {
    /// First radial of an elevation sweep that is not the first sweep in the volume scan
    /// (code 0, "start of new elevation").
    ElevationStart,
    /// A radial within an elevation sweep that is not its first or last radial (code 1,
    /// "intermediate radial data").
    IntermediateRadialData,
    /// Last radial of an elevation sweep (code 2, "end of elevation").
    ElevationEnd,
    /// First radial of the first elevation sweep in a volume scan (code 3, "beginning of volume
    /// scan").
    ScanStart,
    /// Last radial of the last elevation sweep in a volume scan (code 4, "end of volume scan").
    ScanEnd,
    /// First radial of the last elevation sweep in the VCP (code 5, "start of new elevation, last
    /// elevation in VCP").
    ElevationStartVcpFinal,
    /// A radial status code the ICD does not define, carrying the code.
    Unknown(u8),
}

impl RadialStatus {
    /// Classifies a radial status code as it appears on the wire. Codes the ICD does not define
    /// map to [`RadialStatus::Unknown`].
    pub fn from_number(code: u8) -> Self {
        match code {
            0 => RadialStatus::ElevationStart,
            1 => RadialStatus::IntermediateRadialData,
            2 => RadialStatus::ElevationEnd,
            3 => RadialStatus::ScanStart,
            4 => RadialStatus::ScanEnd,
            5 => RadialStatus::ElevationStartVcpFinal,
            other => RadialStatus::Unknown(other),
        }
    }

    /// The radial status code as it appears on the wire.
    pub fn number(self) -> u8 {
        match self {
            RadialStatus::ElevationStart => 0,
            RadialStatus::IntermediateRadialData => 1,
            RadialStatus::ElevationEnd => 2,
            RadialStatus::ScanStart => 3,
            RadialStatus::ScanEnd => 4,
            RadialStatus::ElevationStartVcpFinal => 5,
            RadialStatus::Unknown(code) => code,
        }
    }
}

impl Display for RadialStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RadialStatus::ElevationStart => write!(f, "Elevation Start"),
            RadialStatus::IntermediateRadialData => write!(f, "Intermediate Radial Data"),
            RadialStatus::ElevationEnd => write!(f, "Elevation End"),
            RadialStatus::ScanStart => write!(f, "Scan Start"),
            RadialStatus::ScanEnd => write!(f, "Scan End"),
            RadialStatus::ElevationStartVcpFinal => write!(f, "Elevation Start (Final in VCP)"),
            RadialStatus::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}

#[cfg(feature = "model")]
impl From<RadialStatus> for nexrad_model::data::RadialStatus {
    fn from(status: RadialStatus) -> Self {
        match status {
            RadialStatus::ElevationStart => Self::ElevationStart,
            RadialStatus::IntermediateRadialData => Self::IntermediateRadialData,
            RadialStatus::ElevationEnd => Self::ElevationEnd,
            RadialStatus::ScanStart => Self::ScanStart,
            RadialStatus::ScanEnd => Self::ScanEnd,
            RadialStatus::ElevationStartVcpFinal => Self::ElevationStartVcpFinal,
            RadialStatus::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(feature = "model")]
impl From<nexrad_model::data::RadialStatus> for RadialStatus {
    fn from(status: nexrad_model::data::RadialStatus) -> Self {
        use nexrad_model::data::RadialStatus as Model;

        match status {
            Model::ElevationStart => Self::ElevationStart,
            Model::IntermediateRadialData => Self::IntermediateRadialData,
            Model::ElevationEnd => Self::ElevationEnd,
            Model::ScanStart => Self::ScanStart,
            Model::ScanEnd => Self::ScanEnd,
            Model::ElevationStartVcpFinal => Self::ElevationStartVcpFinal,
            Model::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RadialStatus;

    #[test]
    fn number_round_trips_for_every_code() {
        for code in 0..=u8::MAX {
            assert_eq!(RadialStatus::from_number(code).number(), code);
        }
        assert_eq!(RadialStatus::from_number(3), RadialStatus::ScanStart);
        assert_eq!(RadialStatus::from_number(6), RadialStatus::Unknown(6));
    }

    #[test]
    fn displays_name() {
        assert_eq!(RadialStatus::ScanEnd.to_string(), "Scan End");
        assert_eq!(RadialStatus::Unknown(6).to_string(), "Unknown (6)");
    }

    #[cfg(feature = "model")]
    #[test]
    fn converts_to_and_from_the_model_without_loss() {
        for code in 0..=u8::MAX {
            let decoded = RadialStatus::from_number(code);
            let model = nexrad_model::data::RadialStatus::from(decoded);
            assert_eq!(RadialStatus::from(model), decoded);
        }
    }
}
