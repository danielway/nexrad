/// Number of bits used to store each data moment gate value.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum DataWordSize {
    /// 8 bits per gate.
    EightBit,
    /// 16 bits per gate.
    SixteenBit,
    /// Unknown word size for forward compatibility.
    Unknown(u8),
}
