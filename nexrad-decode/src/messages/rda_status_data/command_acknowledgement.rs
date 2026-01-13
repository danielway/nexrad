/// Acknowledgement of command receipt by RDA system.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CommandAcknowledgement {
    RemoteVCPReceived,
    ClutterBypassMapReceived,
    ClutterCensorZonesReceived,
    RedundantChannelControlCommandAccepted,
}
