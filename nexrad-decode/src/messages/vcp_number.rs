use std::fmt::{Display, Formatter};

/// A volume coverage pattern (VCP) number identifying the radar's scanning strategy.
///
/// This is the single representation of a VCP number across the decoded messages, used by both
/// the digital radar data volume block and the RDA status data message. Unrecognized pattern
/// numbers are preserved in [`VcpNumber::Unknown`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VcpNumber {
    /// VCP 12: Precipitation mode, 14 elevation angles in ~4.5 minutes.
    Vcp12,
    /// VCP 31: Clear air mode, long pulse, 5 elevation angles in ~10 minutes.
    Vcp31,
    /// VCP 32: Clear air mode, short pulse, 5 elevation angles in ~10 minutes.
    Vcp32,
    /// VCP 35: Clear air mode with SZ-2, 9 elevation angles in ~7 minutes.
    Vcp35,
    /// VCP 112: Precipitation mode with SZ-2 and MPDA, ~15 elevation angles in ~5.5 minutes.
    Vcp112,
    /// VCP 212: Precipitation mode with SZ-2, 14-15 elevation angles in ~4.5-5 minutes.
    Vcp212,
    /// VCP 215: General surveillance with SZ-2, 15 elevation angles in ~6 minutes.
    Vcp215,
    /// An unrecognized VCP number for forward compatibility with new scan strategies.
    Unknown(u16),
}

impl VcpNumber {
    /// Creates a `VcpNumber` from a raw pattern number.
    pub fn from_number(number: u16) -> Self {
        match number {
            12 => Self::Vcp12,
            31 => Self::Vcp31,
            32 => Self::Vcp32,
            35 => Self::Vcp35,
            112 => Self::Vcp112,
            212 => Self::Vcp212,
            215 => Self::Vcp215,
            other => Self::Unknown(other),
        }
    }

    /// Returns the raw pattern number.
    pub fn number(self) -> u16 {
        match self {
            Self::Vcp12 => 12,
            Self::Vcp31 => 31,
            Self::Vcp32 => 32,
            Self::Vcp35 => 35,
            Self::Vcp112 => 112,
            Self::Vcp212 => 212,
            Self::Vcp215 => 215,
            Self::Unknown(number) => number,
        }
    }
}

impl From<u16> for VcpNumber {
    fn from(number: u16) -> Self {
        Self::from_number(number)
    }
}

impl From<VcpNumber> for u16 {
    fn from(vcp: VcpNumber) -> Self {
        vcp.number()
    }
}

impl Display for VcpNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VCP {}", self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::VcpNumber;

    #[test]
    fn round_trips_known_and_unknown_numbers() {
        for number in [12, 31, 32, 35, 112, 212, 215, 0, 999] {
            assert_eq!(VcpNumber::from_number(number).number(), number);
            assert_eq!(u16::from(VcpNumber::from(number)), number);
        }
        assert_eq!(VcpNumber::from_number(32), VcpNumber::Vcp32);
        assert_eq!(VcpNumber::from_number(999), VcpNumber::Unknown(999));
    }

    #[test]
    fn displays_pattern_number() {
        assert_eq!(VcpNumber::Vcp215.to_string(), "VCP 215");
        assert_eq!(VcpNumber::Unknown(7).to_string(), "VCP 7");
    }
}
