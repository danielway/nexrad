use std::fmt::{Display, Formatter};

/// A volume coverage pattern (VCP) number, identifying the radar's scanning strategy.
///
/// The RDA/RPG ICD (2620002 series) lists the patterns in its volume coverage pattern appendix.
///
/// The type accepts any `u16`. Associated constants such as [`VcpNumber::VCP_212`] name the
/// patterns 11, 12, 21, 31, 32, 35, 112, 121, 211, 212, 215 and 221. They are ordinary values,
/// so they work as `match` patterns, and a constant added in a later release does not change how
/// any existing value compares or matches. Equality, ordering and hashing use the number.
///
/// With the `serde` feature it serializes as the bare `u16` number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VcpNumber(u16);

impl VcpNumber {
    /// VCP 11.
    pub const VCP_11: VcpNumber = VcpNumber(11);

    /// VCP 12.
    pub const VCP_12: VcpNumber = VcpNumber(12);

    /// VCP 21.
    pub const VCP_21: VcpNumber = VcpNumber(21);

    /// VCP 31.
    pub const VCP_31: VcpNumber = VcpNumber(31);

    /// VCP 32.
    pub const VCP_32: VcpNumber = VcpNumber(32);

    /// VCP 35.
    pub const VCP_35: VcpNumber = VcpNumber(35);

    /// VCP 112.
    pub const VCP_112: VcpNumber = VcpNumber(112);

    /// VCP 121.
    pub const VCP_121: VcpNumber = VcpNumber(121);

    /// VCP 211.
    pub const VCP_211: VcpNumber = VcpNumber(211);

    /// VCP 212.
    pub const VCP_212: VcpNumber = VcpNumber(212);

    /// VCP 215.
    pub const VCP_215: VcpNumber = VcpNumber(215);

    /// VCP 221.
    pub const VCP_221: VcpNumber = VcpNumber(221);

    /// Creates a `VcpNumber` from a pattern number. Every `u16` is accepted.
    pub const fn new(number: u16) -> Self {
        Self(number)
    }

    /// Returns the pattern number.
    pub const fn number(self) -> u16 {
        self.0
    }
}

impl From<u16> for VcpNumber {
    fn from(number: u16) -> Self {
        Self(number)
    }
}

impl From<VcpNumber> for u16 {
    fn from(vcp: VcpNumber) -> Self {
        vcp.0
    }
}

impl Display for VcpNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VCP {}", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for VcpNumber {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VcpNumber {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u16::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::VcpNumber;

    #[test]
    fn round_trips_numbers() {
        for number in [0, 11, 112, 212, 999, u16::MAX] {
            assert_eq!(VcpNumber::new(number).number(), number);
            assert_eq!(u16::from(VcpNumber::from(number)), number);
        }
    }

    #[test]
    fn constants_match_patterns() {
        assert_eq!(VcpNumber::new(212), VcpNumber::VCP_212);
        let label = match VcpNumber::new(35) {
            VcpNumber::VCP_35 => "35",
            VcpNumber::VCP_212 => "212",
            _ => "other",
        };
        assert_eq!(label, "35");
    }

    #[test]
    fn orders_by_number() {
        assert!(VcpNumber::VCP_35 < VcpNumber::VCP_112);
        assert!(VcpNumber::new(7) < VcpNumber::VCP_11);
    }

    #[test]
    fn displays_pattern_number() {
        assert_eq!(VcpNumber::VCP_215.to_string(), "VCP 215");
        assert_eq!(VcpNumber::new(7).to_string(), "VCP 7");
    }
}
