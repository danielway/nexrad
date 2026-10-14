use std::fmt::{Display, Formatter};

/// A volume coverage pattern (VCP) number identifying the radar's scanning strategy.
///
/// This is the single representation of a VCP number across the decoded messages, used by both
/// the digital radar data volume block and the RDA status data message. Unrecognized pattern
/// numbers are preserved in [`VcpNumber::Unknown`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VcpNumber {
    /// VCP 11: Precipitation mode, 14 elevation angles in ~5 minutes, without SZ-2.
    Vcp11,
    /// VCP 12: Precipitation mode, 14 elevation angles in ~4.5 minutes.
    Vcp12,
    /// VCP 21: Precipitation mode, 9 elevation angles in ~6 minutes.
    Vcp21,
    /// VCP 31: Clear air mode, long pulse, 5 elevation angles in ~10 minutes.
    Vcp31,
    /// VCP 32: Clear air mode, short pulse, 5 elevation angles in ~10 minutes.
    Vcp32,
    /// VCP 35: Clear air mode with SZ-2, 9 elevation angles in ~7 minutes.
    Vcp35,
    /// VCP 112: Precipitation mode with SZ-2 and MPDA, ~15 elevation angles in ~5.5 minutes.
    Vcp112,
    /// VCP 121: Precipitation mode with MPDA, using the elevation angles of VCP 21.
    Vcp121,
    /// VCP 211: Precipitation mode with SZ-2, using the elevation angles of VCP 11.
    Vcp211,
    /// VCP 212: Precipitation mode with SZ-2, 14-15 elevation angles in ~4.5-5 minutes.
    Vcp212,
    /// VCP 215: General surveillance with SZ-2, 15 elevation angles in ~6 minutes.
    Vcp215,
    /// VCP 221: Precipitation mode with SZ-2, using the elevation angles of VCP 21.
    Vcp221,
    /// A VCP number without a named variant, for forward compatibility with new scan
    /// strategies. [`from_number`](Self::from_number) never produces this for a number that has
    /// a named variant.
    Unknown(u16),
}

impl VcpNumber {
    /// Creates a `VcpNumber` from a raw pattern number.
    pub fn from_number(number: u16) -> Self {
        match number {
            11 => Self::Vcp11,
            12 => Self::Vcp12,
            21 => Self::Vcp21,
            31 => Self::Vcp31,
            32 => Self::Vcp32,
            35 => Self::Vcp35,
            112 => Self::Vcp112,
            121 => Self::Vcp121,
            211 => Self::Vcp211,
            212 => Self::Vcp212,
            215 => Self::Vcp215,
            221 => Self::Vcp221,
            other => Self::Unknown(other),
        }
    }

    /// Returns the raw pattern number.
    pub fn number(self) -> u16 {
        match self {
            Self::Vcp11 => 11,
            Self::Vcp12 => 12,
            Self::Vcp21 => 21,
            Self::Vcp31 => 31,
            Self::Vcp32 => 32,
            Self::Vcp35 => 35,
            Self::Vcp112 => 112,
            Self::Vcp121 => 121,
            Self::Vcp211 => 211,
            Self::Vcp212 => 212,
            Self::Vcp215 => 215,
            Self::Vcp221 => 221,
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
        for number in [11, 12, 21, 31, 32, 35, 112, 121, 211, 212, 215, 221, 0, 999] {
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
