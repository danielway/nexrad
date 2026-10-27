//! Assembly of decoded messages into `nexrad_model` types.
//!
//! [`ScanBuilder`] collects decoded [`Message`]s and produces a complete
//! [`Scan`]: digital radar data messages (types 1 and 31) become radials,
//! the first volume coverage pattern message (type 5) becomes the scan's coverage pattern, and
//! the first volume data block found identifies the radar site.

use crate::messages::{Message, MessageContents};
use crate::result::{Error, Result};
use nexrad_model::data::{Radial, Scan, Sweep, VolumeCoveragePattern};
use nexrad_model::meta::Site;

/// Accumulates decoded messages into a [`Scan`].
///
/// Feed messages in collection order with [`add_message`](Self::add_message) or
/// [`add_messages`](Self::add_messages), then call [`build`](Self::build). Independently built
/// builders (for example, one per LDM record decoded in parallel) can be combined in order with
/// [`merge`](Self::merge).
///
/// ```no_run
/// use nexrad_decode::messages::decode_messages;
/// use nexrad_decode::model::ScanBuilder;
///
/// # fn run(records: Vec<Vec<u8>>) -> nexrad_decode::Result<()> {
/// let mut builder = ScanBuilder::new();
/// for record in &records {
///     builder.add_messages(decode_messages(record)?)?;
/// }
/// let scan = builder.build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct ScanBuilder {
    coverage_pattern: Option<VolumeCoveragePattern>,
    site: Option<Site>,
    radials: Vec<Radial>,
}

impl ScanBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a decoded message to the scan.
    ///
    /// Digital radar data messages (types 1 and 31) are converted to radials, and the first type
    /// 31 message carrying a volume data block records the scan's site. The first volume
    /// coverage pattern message is kept; later ones are ignored, as are all other message types.
    ///
    /// # Errors
    ///
    /// Returns an error if a radar data message cannot be converted to a radial or its site
    /// metadata is invalid, or a volume coverage pattern message cannot be converted to the
    /// model.
    pub fn add_message(&mut self, message: Message<'_>) -> Result<()> {
        match message.into_contents() {
            MessageContents::DigitalRadarData(message) => {
                if self.site.is_none() {
                    self.site = message.site()?;
                }
                self.radials.push(message.into_radial()?);
            }
            MessageContents::DigitalRadarDataLegacy(message) => {
                self.radials.push(message.into_radial()?);
            }
            MessageContents::RdaVolumeCoveragePattern(message)
                if self.coverage_pattern.is_none() =>
            {
                self.coverage_pattern = Some(message.to_model()?);
            }
            _ => {}
        }
        Ok(())
    }

    /// Adds each of the given messages in order. See [`add_message`](Self::add_message).
    ///
    /// # Errors
    ///
    /// Returns the error for the first message that [`add_message`](Self::add_message) rejects.
    pub fn add_messages<'a>(
        &mut self,
        messages: impl IntoIterator<Item = Message<'a>>,
    ) -> Result<()> {
        for message in messages {
            self.add_message(message)?;
        }
        Ok(())
    }

    /// Appends another builder's radials after this builder's, keeping this builder's coverage
    /// pattern and site when set and otherwise taking the other's.
    pub fn merge(&mut self, other: ScanBuilder) {
        if self.coverage_pattern.is_none() {
            self.coverage_pattern = other.coverage_pattern;
        }
        if self.site.is_none() {
            self.site = other.site;
        }
        self.radials.extend(other.radials);
    }

    /// Builds the scan, grouping radials into sweeps by elevation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingCoveragePattern`] if no volume coverage pattern message was added.
    pub fn build(self) -> Result<Scan> {
        let coverage_pattern = self.coverage_pattern.ok_or(Error::MissingCoveragePattern)?;
        let sweeps = Sweep::split_by_elevation_number(self.radials);
        Ok(match self.site {
            Some(site) => Scan::with_site(site, coverage_pattern, sweeps),
            None => Scan::new(coverage_pattern, sweeps),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::decode_messages;

    const VCP_DATA: &[u8] = include_bytes!("../tests/data/messages/volume_coverage_pattern.bin");
    const RADAR_DATA: &[u8] = include_bytes!("../tests/data/messages/digital_radar_data_full.bin");
    const LEGACY_RADAR_DATA: &[u8] =
        include_bytes!("../tests/data/messages/digital_radar_data_legacy.bin");

    fn single_message(data: &[u8]) -> Message<'_> {
        let mut messages = decode_messages(data).expect("decodes successfully");
        assert_eq!(messages.len(), 1, "expected exactly one message");
        messages.remove(0)
    }

    fn radials(scan: &Scan) -> Vec<&Radial> {
        scan.sweeps()
            .iter()
            .flat_map(|sweep| sweep.radials())
            .collect()
    }

    #[test]
    fn vcp_to_model() {
        let message = single_message(VCP_DATA);
        let MessageContents::RdaVolumeCoveragePattern(vcp) = message.contents() else {
            panic!("expected a volume coverage pattern message");
        };

        let model = vcp.to_model().unwrap();
        assert_eq!(model.pattern_number().number(), 212);
        assert_eq!(model.version(), vcp.header().version());
        assert_eq!(model.pulse_width(), nexrad_model::data::PulseWidth::Short);
        assert!(model.mrle_enabled());
        assert_eq!(model.mrle_cut_count(), 3);
        assert_eq!(model.elevation_cut_count(), vcp.elevations().len());

        let first = &model.elevation_cuts()[0];
        assert_eq!(first.elevation_angle_degrees(), 0.483_398_44);
        assert_eq!(
            first.channel_configuration(),
            nexrad_model::data::ChannelConfiguration::Sz2Phase
        );
        assert_eq!(
            first.waveform_type(),
            nexrad_model::data::WaveformType::ContiguousSurveillance
        );
        assert_eq!(
            model.elevation_cuts()[1].waveform_type(),
            nexrad_model::data::WaveformType::ContiguousDopplerWithAmbiguityResolution
        );
    }

    #[test]
    fn radar_data_site() {
        let message = single_message(RADAR_DATA);
        let MessageContents::DigitalRadarData(radar) = message.contents() else {
            panic!("expected a digital radar data message");
        };

        let site = radar
            .site()
            .expect("site is valid")
            .expect("message has a volume data block");
        assert_eq!(site.identifier(), "KDMX");
        assert_eq!(site.latitude(), 41.7312_f32 as f64);
        assert_eq!(site.longitude(), -93.72287_f32 as f64);
        assert_eq!(site.site_elevation_meters(), 299);
        assert_eq!(site.feedhorn_height_meters(), 34);
    }

    #[test]
    fn model_errors_convert_into_decode_errors() {
        let error: Error = nexrad_model::result::Error::InvalidSite {
            parameter: "latitude",
            reason: "must be finite and within -90.0..=90.0",
        }
        .into();
        assert!(matches!(
            error,
            Error::Model(nexrad_model::result::Error::InvalidSite {
                parameter: "latitude",
                ..
            })
        ));
    }

    #[test]
    fn build_without_coverage_pattern_fails() {
        let mut builder = ScanBuilder::new();
        builder
            .add_message(single_message(RADAR_DATA))
            .expect("adds radar data");

        assert!(matches!(
            builder.build(),
            Err(Error::MissingCoveragePattern)
        ));
    }

    #[test]
    fn build_includes_modern_and_legacy_radials() {
        let modern = single_message(RADAR_DATA);
        let legacy = single_message(LEGACY_RADAR_DATA);
        let (expected_modern, expected_legacy) = match (modern.contents(), legacy.contents()) {
            (
                MessageContents::DigitalRadarData(modern),
                MessageContents::DigitalRadarDataLegacy(legacy),
            ) => (
                modern.radial().expect("modern radial"),
                legacy.radial().expect("legacy radial"),
            ),
            _ => panic!("unexpected message types"),
        };

        let mut builder = ScanBuilder::new();
        builder
            .add_messages([single_message(VCP_DATA), modern, legacy])
            .expect("adds messages");
        let scan = builder.build().expect("builds scan");

        assert_eq!(scan.coverage_pattern_number().number(), 212);
        assert_eq!(scan.site().map(|site| site.identifier()), Some("KDMX"));
        assert_eq!(radials(&scan), [&expected_modern, &expected_legacy]);
    }

    #[test]
    fn merge_appends_radials_and_keeps_first_metadata() {
        let mut first = ScanBuilder::new();
        first
            .add_message(single_message(LEGACY_RADAR_DATA))
            .expect("adds legacy radar data");

        let mut second = ScanBuilder::new();
        second
            .add_messages([single_message(VCP_DATA), single_message(RADAR_DATA)])
            .expect("adds messages");

        let mut expected = ScanBuilder::new();
        expected
            .add_messages([
                single_message(LEGACY_RADAR_DATA),
                single_message(VCP_DATA),
                single_message(RADAR_DATA),
            ])
            .expect("adds messages");

        first.merge(second);
        let merged = first.build().expect("builds merged scan");
        assert_eq!(merged, expected.build().expect("builds expected scan"));
        assert_eq!(radials(&merged).len(), 2);
        assert!(merged.site().is_some());
    }
}
