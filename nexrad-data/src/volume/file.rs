use crate::volume::{split_compressed_records, Header, Record};
use std::fmt::Debug;
use zerocopy::Ref;

/// Gzip magic bytes (RFC 1952).
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// A NEXRAD Archive II volume data file.
///
/// Older NEXRAD archives (pre-~2016) may be gzip-compressed. Call
/// [`decompress`](Self::decompress) to inflate before accessing records.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct File(Vec<u8>);

impl File {
    /// Creates a new Archive II volume file with the provided data.
    ///
    /// The data is stored as-is. Call [`decompress`](Self::decompress) before
    /// accessing records if the file may be gzip-compressed.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Whether this file's data is gzip-compressed.
    ///
    /// Some older NEXRAD archive files (pre-~2016) are gzip-wrapped. These must
    /// be decompressed before records can be accessed.
    pub fn compressed(&self) -> bool {
        self.0.len() >= 2 && self.0[..2] == GZIP_MAGIC
    }

    /// Decompresses a gzip-compressed volume file.
    ///
    /// Returns a new `File` containing the decompressed data. If the file is not
    /// gzip-compressed, returns it unchanged. Returns an error only if decompression
    /// fails.
    pub fn decompress(self) -> crate::result::Result<File> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        if !self.compressed() {
            return Ok(self);
        }

        let mut decoder = GzDecoder::new(self.0.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        Ok(File(decompressed))
    }

    /// The file's raw data.
    ///
    /// If the file is gzip-compressed, this returns the compressed bytes.
    /// Call [`decompress`](Self::decompress) first to get the decompressed content.
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// The file's decoded Archive II volume header.
    ///
    /// Returns `None` if the header cannot be parsed (e.g. if the file is still
    /// gzip-compressed).
    pub fn header(&self) -> Option<&Header> {
        Ref::<_, Header>::from_prefix(self.0.as_slice())
            .ok()
            .map(|(header, _rest)| Ref::into_ref(header))
    }

    /// The file's LDM records.
    ///
    /// Returns an error if the file is gzip-compressed (call [`decompress`](Self::decompress)
    /// first) or if the record data is truncated or malformed.
    pub fn records(&self) -> crate::result::Result<Vec<Record<'_>>> {
        if self.compressed() {
            return Err(crate::result::Error::CompressedFile);
        }
        split_compressed_records(&self.0[size_of::<Header>()..])
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    #[cfg(feature = "model")]
    pub fn scan(&self) -> crate::result::Result<nexrad_model::data::Scan> {
        use crate::result::Error;
        use nexrad_decode::messages::volume_coverage_pattern as vcp;
        use nexrad_decode::messages::MessageContents;
        use nexrad_model::data::{
            ChannelConfiguration, ElevationCut, PulseWidth, Radial, Scan, Sweep, VcpNumber,
            VolumeCoveragePattern, WaveformType,
        };

        // Site location extracted from a DigitalRadarData volume data block.
        struct SiteLocation {
            latitude: f32,
            longitude: f32,
            site_height: i16,
            tower_height: u16,
        }

        let records = self.records()?;

        let process_record = |record: Record<'_>| -> crate::result::Result<(
            Option<vcp::Message<'static>>,
            Vec<Radial>,
            Option<SiteLocation>,
        )> {
            let record = if record.compressed() {
                record.decompress()?
            } else {
                Record::new(record.data().to_vec())
            };

            let mut vcp = None;
            let mut radials = Vec::new();
            let mut site_location = None;
            for message in record.messages()? {
                match message.into_contents() {
                    MessageContents::DigitalRadarData(m) => {
                        if site_location.is_none() {
                            if let Some(vol_block) = m.volume_data_block() {
                                site_location = Some(SiteLocation {
                                    latitude: vol_block.inner().latitude_raw(),
                                    longitude: vol_block.inner().longitude_raw(),
                                    site_height: vol_block.inner().site_height_raw(),
                                    tower_height: vol_block.inner().tower_height_raw(),
                                });
                            }
                        }
                        radials.push(m.into_radial()?);
                    }
                    MessageContents::VolumeCoveragePattern(m) if vcp.is_none() => {
                        vcp = Some(m.into_owned());
                    }
                    _ => {}
                }
            }
            Ok((vcp, radials, site_location))
        };

        #[cfg(feature = "parallel")]
        let results: Vec<_> = {
            use rayon::prelude::*;
            records
                .into_par_iter()
                .map(process_record)
                .collect::<crate::result::Result<Vec<_>>>()?
        };

        #[cfg(not(feature = "parallel"))]
        let results: Vec<_> = records
            .into_iter()
            .map(process_record)
            .collect::<crate::result::Result<Vec<_>>>()?;

        let mut coverage_pattern_message = None;
        let mut radials = Vec::new();
        let mut site_location = None;
        for (vcp, r, loc) in results {
            if coverage_pattern_message.is_none() {
                coverage_pattern_message = vcp;
            }
            if site_location.is_none() {
                site_location = loc;
            }
            radials.extend(r);
        }

        // Convert the VCP message to the model representation
        let vcp_msg = coverage_pattern_message.ok_or(Error::MissingCoveragePattern)?;
        let header = vcp_msg.header();

        let pulse_width = match header.pulse_width() {
            vcp::PulseWidth::Short => PulseWidth::Short,
            vcp::PulseWidth::Long => PulseWidth::Long,
            vcp::PulseWidth::Unknown => PulseWidth::Unknown,
        };

        let elevation_cuts: Vec<ElevationCut> = vcp_msg
            .elevations()
            .iter()
            .map(|elev| {
                let channel_config = match elev.channel_configuration() {
                    vcp::ChannelConfiguration::ConstantPhase => ChannelConfiguration::ConstantPhase,
                    vcp::ChannelConfiguration::RandomPhase => ChannelConfiguration::RandomPhase,
                    vcp::ChannelConfiguration::Sz2Phase => ChannelConfiguration::Sz2Phase,
                    vcp::ChannelConfiguration::UnknownPhase => ChannelConfiguration::Unknown,
                };

                let waveform = match elev.waveform_type() {
                    vcp::WaveformType::ContiguousSurveillance => {
                        WaveformType::ContiguousSurveillance
                    }
                    vcp::WaveformType::ContiguousDopplerWithAmbiguityResolution => {
                        WaveformType::ContiguousDopplerWithAmbiguityResolution
                    }
                    vcp::WaveformType::ContiguousDopplerWithoutAmbiguityResolution => {
                        WaveformType::ContiguousDopplerWithoutAmbiguityResolution
                    }
                    vcp::WaveformType::Batch => WaveformType::Batch,
                    vcp::WaveformType::StaggeredPulsePair => WaveformType::StaggeredPulsePair,
                    vcp::WaveformType::Unknown => WaveformType::Unknown,
                };

                ElevationCut::builder(elev.elevation_angle(), waveform, channel_config)
                    .azimuth_rate_degrees_per_second(elev.azimuth_rate())
                    .super_resolution_half_degree_azimuth(
                        elev.super_resolution_half_degree_azimuth(),
                    )
                    .super_resolution_quarter_km_reflectivity(
                        elev.super_resolution_quarter_km_reflectivity(),
                    )
                    .super_resolution_doppler_to_300km(elev.super_resolution_doppler_to_300km())
                    .super_resolution_dual_pol_to_300km(elev.super_resolution_dual_pol_to_300km())
                    .surveillance_prf_number(elev.surveillance_prf_number())
                    .surveillance_prf_pulse_count(elev.surveillance_prf_pulse_count_radial())
                    .reflectivity_threshold_db(elev.reflectivity_threshold())
                    .velocity_threshold_db(elev.velocity_threshold())
                    .spectrum_width_threshold_db(elev.spectrum_width_threshold())
                    .differential_reflectivity_threshold_db(
                        elev.differential_reflectivity_threshold(),
                    )
                    .differential_phase_threshold_db(elev.differential_phase_threshold())
                    .correlation_coefficient_threshold_db(elev.correlation_coefficient_threshold())
                    .is_sails_cut(elev.is_sails_cut())
                    .sails_sequence_number(elev.sails_sequence_number())
                    .is_mrle_cut(elev.is_mrle_cut())
                    .mrle_sequence_number(elev.mrle_sequence_number())
                    .is_mpda_cut(elev.is_mpda_cut())
                    .is_base_tilt_cut(elev.is_base_tilt_cut())
                    .build()
            })
            .collect();

        let coverage_pattern =
            VolumeCoveragePattern::builder(VcpNumber::from_number(header.pattern_number()))
                .version(header.version())
                .doppler_velocity_resolution(header.doppler_velocity_resolution())
                .pulse_width(pulse_width)
                .sails_enabled(header.is_sails_vcp())
                .sails_cuts(header.number_of_sails_cuts())
                .mrle_enabled(header.is_mrle_vcp())
                .mrle_cuts(header.number_of_mrle_cuts())
                .mpda_enabled(header.is_mpda_vcp())
                .base_tilt_enabled(header.is_base_tilt_vcp())
                .base_tilt_count(header.number_of_base_tilts())
                .sequence_active(header.vcp_sequencing_sequence_active())
                .truncated(header.vcp_sequencing_truncated())
                .elevation_cuts(elevation_cuts)
                .build();

        // Build the site from the volume header ICAO and the location from radar messages.
        let site = site_location.map(|loc| {
            let identifier = self
                .header()
                .and_then(|h| h.icao_of_radar())
                .map(|icao| {
                    let bytes = icao.as_bytes();
                    let mut id = [0u8; 4];
                    let len = bytes.len().min(4);
                    id[..len].copy_from_slice(&bytes[..len]);
                    id
                })
                .unwrap_or([0u8; 4]);

            nexrad_model::meta::Site::builder(identifier, loc.latitude as f64, loc.longitude as f64)
                .height_meters(loc.site_height)
                .tower_height_meters(loc.tower_height)
                .build()
        });

        let sweeps = Sweep::from_radials(radials);
        match site {
            Some(site) => Ok(Scan::with_site(site, coverage_pattern, sweeps)),
            None => Ok(Scan::new(coverage_pattern, sweeps)),
        }
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("File");
        debug.field("data.len()", &self.data().len());
        debug.field("header", &self.header());

        #[cfg(feature = "model")]
        debug.field(
            "records.len()",
            &self.records().map(|records| records.len()),
        );

        debug.finish()
    }
}
