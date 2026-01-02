/// The value for a data moment/radial, gate, and product. The value may be a floating-point number
/// or a special case such as "below threshold" or "range folded".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaledMomentValue {
    /// The converted floating-point representation of the data moment value for a gate.
    Value(f32),
    /// The value for this gate was below the signal threshold.
    BelowThreshold,
    /// The value for this gate exceeded the maximum unambiguous range.
    RangeFolded,
}
