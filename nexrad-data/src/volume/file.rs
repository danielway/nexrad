use crate::result::Result;
use crate::volume::{split_compressed_records, Header, Record};

/// A NEXRAD Archive II volume data file.
pub struct File(Vec<u8>);

impl File {
    /// Creates a new Archive II volume file with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// The file's encoded and compressed data.
    pub fn data(&self) -> &Vec<u8> {
        &self.0
    }

    /// The file's decoded Archive II volume header.
    pub fn header(&self) -> Result<Header> {
        Header::deserialize(&mut self.0.as_slice())
    }

    /// The file's LDM records.
    pub fn records(&self) -> Vec<Record> {
        split_compressed_records(&self.0[size_of::<Header>()..])
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    #[cfg(feature = "decode")]
    pub fn scan(&self) -> Result<nexrad_model::data::Scan> {
        use crate::result::Error;
        use nexrad_model::data::{Scan, Sweep};

        let mut sweeps = Vec::new();
        let mut current_sweep: Option<Sweep> = None;

        for mut record in self.records() {
            if record.compressed() {
                record = record.decompress()?;
            }

            for sweep in record.sweeps()? {
                let elevation = current_sweep.as_ref().map(|sweep| sweep.elevation_number());
                if elevation != Some(sweep.elevation_number()) {
                    if let Some(current_sweep) = current_sweep.take() {
                        sweeps.push(current_sweep);
                    }
                }

                if let Some(old_sweep) = current_sweep.take() {
                    current_sweep = Some(old_sweep.merge(sweep)?);
                } else {
                    current_sweep = Some(sweep);
                }
            }
        }

        if let Some(current_sweep) = current_sweep.take() {
            sweeps.push(current_sweep);
        }

        Ok(Scan::new(
            1, // TODO: coverage_pattern.ok_or(Error::MissingCoveragePattern)?
            sweeps,
        ))
    }
}
