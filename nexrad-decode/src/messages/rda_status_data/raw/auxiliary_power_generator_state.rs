/// The possible RDA system auxiliary power generator states.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AuxiliaryPowerGeneratorState {
    SwitchedToAuxiliaryPower,
    UtilityPowerAvailable,
    GeneratorOn,
    TransferSwitchSetToManual,
    CommandedSwitchover,
}
