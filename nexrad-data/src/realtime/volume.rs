use std::fmt::Display;

/// Represents a volume index in the AWS S3 bucket containing NEXRAD chunk data.
#[derive(Clone, Copy, Debug)]
pub struct Volume(usize);

impl Volume {
    pub(crate) fn new(volume: usize) -> Self {
        Self(volume)
    }

    /// This volume's index.
    pub fn number(&self) -> usize {
        self.0
    }
}

impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
