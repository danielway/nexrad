/// An RDA alarm message definition to be referenced by an RDA status data message.
pub struct Message {
    code: u16,
    state: Option<State>,
    alarm_type: Option<AlarmType>,
    device: Option<Device>,
    sample: Option<u8>,
    message: &'static str,
}

impl Message {
    pub(crate) fn new(
        code: u16,
        state: Option<State>,
        alarm_type: Option<AlarmType>,
        device: Option<Device>,
        sample: Option<u8>,
        message: &'static str,
    ) -> Self {
        Self {
            code,
            state,
            alarm_type,
            device,
            sample,
            message,
        }
    }
}

/// The status of the RDA as a result of the alarm.
pub enum State {
    MaintenanceMandatory,
    MaintenanceRequired,
    Inoperative,
    /// Alarm not specifically tied to state change.
    Secondary,
}

/// The different classifications of alarms.
pub enum AlarmType {
    /// Alarm failed consecutively enough times to meet the alarm reporting count/sample threshold.
    EdgeDetected,
    /// Alarm reported each time the condition is met.
    Occurrence,
    /// Alarm reported at most once every 15 minutes when the condition is met.
    FilteredOccurrence,
}

/// The hardware device area where the alarm originated.
pub enum Device {
    Control,
    Pedestal,
    Receiver,
    SignalProcessor,
    Communications,
    TowerUtilities,
    Transmitter,
}
