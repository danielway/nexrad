use crate::messages::primitive_aliases::Code2;
use std::fmt::Debug;

/// The RDA system's active alarm types.
pub struct Summary(Code2);

impl Summary {
    pub(crate) fn new(code: Code2) -> Self {
        Self(code)
    }

    /// Whether no alarms are active.
    pub fn none(&self) -> bool {
        self.0 == 0
    }

    /// Whether the tower/utilities alarm is active.
    pub fn tower_utilities(&self) -> bool {
        self.0 & 0b0001 != 0
    }

    /// Whether the pedestal alarm is active.
    pub fn pedestal(&self) -> bool {
        self.0 & 0b0010 != 0
    }

    /// Whether the transmitter alarm is active.
    pub fn transmitter(&self) -> bool {
        self.0 & 0b0100 != 0
    }

    /// Whether the receiver alarm is active.
    pub fn receiver(&self) -> bool {
        self.0 & 0b1000 != 0
    }

    /// Whether the RDA control alarm is active.
    pub fn rda_control(&self) -> bool {
        self.0 & 0b10000 != 0
    }

    /// Whether the communication alarm is active.
    pub fn communication(&self) -> bool {
        self.0 & 0b100000 != 0
    }

    /// Whether the signal processor alarm is active.
    pub fn signal_processor(&self) -> bool {
        self.0 & 0b1000000 != 0
    }
}

impl Debug for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Summary")
            .field("none", &self.none())
            .field("tower_utilities", &self.tower_utilities())
            .field("pedestal", &self.pedestal())
            .field("transmitter", &self.transmitter())
            .field("receiver", &self.receiver())
            .field("rda_control", &self.rda_control())
            .field("communication", &self.communication())
            .field("signal_processor", &self.signal_processor())
            .finish()
    }
}
