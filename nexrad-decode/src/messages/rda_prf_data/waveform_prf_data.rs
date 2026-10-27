use crate::messages::volume_coverage_pattern::WaveformType;

/// PRF (Pulse Repetition Frequency) data for a single waveform type.
///
/// Each waveform section contains a waveform type identifier followed by the set of PRF values
/// used for that waveform type. The PRF values are raw Integer4 values scaled by 0.001 to produce
/// the PRF in Hz.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WaveformPrfData {
    /// The waveform type code: 1 = Contiguous Surveillance, 2 = Contiguous Doppler with
    /// Ambiguity Resolution, 5 = Staggered Pulse Pair.
    waveform_type: u16,

    /// The raw PRF values (Integer4). Each value is scaled by 0.001 to get the PRF in Hz.
    prf_values: Vec<u32>,
}

impl WaveformPrfData {
    /// Creates a new waveform PRF data entry.
    pub(crate) fn new(waveform_type: u16, prf_values: Vec<u32>) -> Self {
        WaveformPrfData {
            waveform_type,
            prf_values,
        }
    }

    /// The waveform type code: 1 = Contiguous Surveillance, 2 = Contiguous Doppler with
    /// Ambiguity Resolution, 5 = Staggered Pulse Pair (raw value). See
    /// [`waveform_type()`](Self::waveform_type) for the decoded value.
    pub fn waveform_type_raw(&self) -> u16 {
        self.waveform_type
    }

    /// The waveform type. A code that does not fit in a `u8` (only possible in malformed data)
    /// is reported as [`WaveformType::Unknown(u8::MAX)`](WaveformType::Unknown).
    pub fn waveform_type(&self) -> WaveformType {
        WaveformType::from_number(u8::try_from(self.waveform_type).unwrap_or(u8::MAX))
    }

    /// The raw PRF values as Integer4 values. Each value is scaled by 0.001 to get the PRF in Hz.
    pub fn prf_values(&self) -> &[u32] {
        &self.prf_values
    }

    /// The number of PRF values for this waveform type.
    pub fn prf_count(&self) -> usize {
        self.prf_values.len()
    }
}
