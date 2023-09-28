//!
//! Struct definitions for decoded NEXRAD Level II data structures.
//!

use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// A decoded NEXRAD WSR-88D data file including sweep data.
pub struct DataFile {
    volume_header: VolumeHeaderRecord,
    elevation_scans: HashMap<u8, Vec<Message31>>,
}

impl DataFile {
    /// Create a new data file for the specified header with no sweep data.
    pub(crate) fn new(file_header: VolumeHeaderRecord) -> Self {
        Self {
            volume_header: file_header,
            elevation_scans: HashMap::new(),
        }
    }

    /// The volume/file header information.
    pub fn volume_header(&self) -> &VolumeHeaderRecord {
        &self.volume_header
    }

    /// Scan data grouped by elevation.
    pub fn elevation_scans(&self) -> &HashMap<u8, Vec<Message31>> {
        &self.elevation_scans
    }

    /// Scan data grouped by elevation.
    pub(crate) fn elevation_scans_mut(&mut self) -> &mut HashMap<u8, Vec<Message31>> {
        &mut self.elevation_scans
    }
}

/// NEXRAD data volume/file header.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeHeaderRecord {
    filename: [u8; 12],
    file_date: u32,
    file_time: u32,
    radar_id: [u8; 4],
}

impl VolumeHeaderRecord {
    /// Filename of the archive.
    pub fn filename(&self) -> &[u8; 12] {
        &self.filename
    }

    /// Modified Julian date of the file.
    pub fn file_date(&self) -> u32 {
        self.file_date
    }

    /// Milliseconds of day since midnight of the file.
    pub fn file_time(&self) -> u32 {
        self.file_time
    }

    /// ICAO radar identifier in ASCII.
    pub fn radar_id(&self) -> &[u8; 4] {
        &self.radar_id
    }
}

/// A NEXRAD volume message header indicating its type and size to be decoded.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageHeader {
    rpg: [u8; 12],
    msg_size: u16,
    channel: u8,
    msg_type: u8,
    id_seq: u16,
    msg_date: u16,
    msg_time: u32,
    num_segs: u16,
    seg_num: u16,
}

impl MessageHeader {
    /// 12 bytes inserted by RPG Communications Mgr. Ignored.
    pub fn rpg(&self) -> &[u8; 12] {
        &self.rpg
    }

    /// Message size for this segment, in halfwords.
    pub fn msg_size(&self) -> u16 {
        self.msg_size
    }

    /// RDA Redundant Channel
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// Message type. For example, 31.
    pub fn msg_type(&self) -> u8 {
        self.msg_type
    }

    /// Msg seq num = 0 to 7FFF, then roll over to 0.
    pub fn id_seq(&self) -> u16 {
        self.id_seq
    }

    /// Modified Julian date from 1/1/70.
    pub fn msg_date(&self) -> u16 {
        self.msg_date
    }

    /// Packet generation time in ms past midnight.
    pub fn msg_time(&self) -> u32 {
        self.msg_time
    }

    /// Number of segments for this message.
    pub fn num_segs(&self) -> u16 {
        self.num_segs
    }

    /// Number of this segment.
    pub fn seg_num(&self) -> u16 {
        self.seg_num
    }
}

/// Structured data for message type 31.
pub struct Message31 {
    header: Message31Header,
    volume_data: Option<VolumeData>,
    elevation_data: Option<ElevationData>,
    radial_data: Option<RadialData>,
    reflectivity_data: Option<DataMoment>,
    velocity_data: Option<DataMoment>,
    sw_data: Option<DataMoment>,
    zdr_data: Option<DataMoment>,
    phi_data: Option<DataMoment>,
    rho_data: Option<DataMoment>,
    cfp_data: Option<DataMoment>,
}

impl Message31 {
    /// Create a new message 31 structure with just the header to start.
    pub(crate) fn new(header: Message31Header) -> Self {
        Self {
            header,
            volume_data: None,
            elevation_data: None,
            radial_data: None,
            reflectivity_data: None,
            velocity_data: None,
            sw_data: None,
            zdr_data: None,
            phi_data: None,
            rho_data: None,
            cfp_data: None,
        }
    }

    /// The message 31 header.
    pub fn header(&self) -> &Message31Header {
        &self.header
    }

    /// The volume data block.
    pub fn volume_data(&self) -> Option<&VolumeData> {
        self.volume_data.as_ref()
    }

    /// The elevation data block.
    pub fn elevation_data(&self) -> Option<&ElevationData> {
        self.elevation_data.as_ref()
    }

    /// The radial data block.
    pub fn radial_data(&self) -> Option<&RadialData> {
        self.radial_data.as_ref()
    }

    /// The reflectivity data block.
    pub fn reflectivity_data(&self) -> Option<&DataMoment> {
        self.reflectivity_data.as_ref()
    }

    /// The velocity data block.
    pub fn velocity_data(&self) -> Option<&DataMoment> {
        self.velocity_data.as_ref()
    }

    /// The spectrum width data block.
    pub fn sw_data(&self) -> Option<&DataMoment> {
        self.sw_data.as_ref()
    }

    /// The differential reflectivity data block.
    pub fn zdr_data(&self) -> Option<&DataMoment> {
        self.zdr_data.as_ref()
    }

    /// The differential phase data block.
    pub fn phi_data(&self) -> Option<&DataMoment> {
        self.phi_data.as_ref()
    }

    /// The correlation coefficient data block.
    pub fn rho_data(&self) -> Option<&DataMoment> {
        self.rho_data.as_ref()
    }

    /// The clutter filter power data block.
    pub fn cfp_data(&self) -> Option<&DataMoment> {
        self.cfp_data.as_ref()
    }

    /// Set the volume data block.
    pub(crate) fn set_volume_data(&mut self, volume_data: VolumeData) {
        self.volume_data = Some(volume_data);
    }

    /// Set the elevation data block.
    pub(crate) fn set_elevation_data(&mut self, elevation_data: ElevationData) {
        self.elevation_data = Some(elevation_data);
    }

    /// Set the radial data block.
    pub(crate) fn set_radial_data(&mut self, radial_data: RadialData) {
        self.radial_data = Some(radial_data);
    }

    /// Set the reflectivity data block.
    pub(crate) fn set_reflectivity_data(&mut self, reflectivity_data: DataMoment) {
        self.reflectivity_data = Some(reflectivity_data);
    }

    /// Set the velocity data block.
    pub(crate) fn set_velocity_data(&mut self, velocity_data: DataMoment) {
        self.velocity_data = Some(velocity_data);
    }

    /// Set the spectrum width data block.
    pub(crate) fn set_sw_data(&mut self, sw_data: DataMoment) {
        self.sw_data = Some(sw_data);
    }

    /// Set the differential reflectivity data block.
    pub(crate) fn set_zdr_data(&mut self, zdr_data: DataMoment) {
        self.zdr_data = Some(zdr_data);
    }

    /// Set the differential phase data block.
    pub(crate) fn set_phi_data(&mut self, phi_data: DataMoment) {
        self.phi_data = Some(phi_data);
    }

    /// Set the correlation coefficient data block.
    pub(crate) fn set_rho_data(&mut self, rho_data: DataMoment) {
        self.rho_data = Some(rho_data);
    }

    /// Set the clutter filter power data block.
    pub(crate) fn set_cfp_data(&mut self, cfp_data: DataMoment) {
        self.cfp_data = Some(cfp_data);
    }
}

/// Header for message type 31.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Message31Header {
    radar_id: [u8; 4],
    ray_time: u32,
    ray_date: u16,
    azm_num: u16,
    azm: f32,
    compression_code: u8,
    spare: u8,
    radial_len: u16,
    azm_res: u8,
    radial_status: u8,
    elev_num: u8,
    sector_cut_num: u8,
    elev: f32,
    radial_spot_blanking: u8,
    azm_indexing_mode: u8,
    data_block_count: u16,
}

impl Message31Header {
    /// Radar site identifier.
    pub fn radar_id(&self) -> &[u8; 4] {
        &self.radar_id
    }

    /// Data collection time in milliseconds past midnight GMT.
    pub fn ray_time(&self) -> u32 {
        self.ray_time
    }

    /// Julian date - 2440586.5 (1/01/1970).
    pub fn ray_date(&self) -> u16 {
        self.ray_date
    }

    /// Radial number within elevation scan.
    pub fn azm_num(&self) -> u16 {
        self.azm_num
    }

    /// Azimuth angle in degrees (0 to 359.956055).
    pub fn azm(&self) -> f32 {
        self.azm
    }

    /// 0 = uncompressed, 1 = BZIP2, 2 = zlib.
    pub fn compression_code(&self) -> u8 {
        self.compression_code
    }

    /// For word alignment.
    pub fn spare(&self) -> u8 {
        self.spare
    }

    /// Radial length in bytes, including data header block.
    pub fn radial_len(&self) -> u16 {
        self.radial_len
    }

    /// Azimuthal resolution.
    pub fn azm_res(&self) -> u8 {
        self.azm_res
    }

    /// Radial status.
    pub fn radial_status(&self) -> u8 {
        self.radial_status
    }

    /// Elevation number.
    pub fn elev_num(&self) -> u8 {
        self.elev_num
    }

    /// Sector cut number.
    pub fn sector_cut_num(&self) -> u8 {
        self.sector_cut_num
    }

    /// Elevation angle in degrees (-7.0 to 70.0).
    pub fn elev(&self) -> f32 {
        self.elev
    }

    /// Radial spot blanking.
    pub fn radial_spot_blanking(&self) -> u8 {
        self.radial_spot_blanking
    }

    /// Azimuth indexing mode.
    pub fn azm_indexing_mode(&self) -> u8 {
        self.azm_indexing_mode
    }

    /// Data block count.
    pub fn data_block_count(&self) -> u16 {
        self.data_block_count
    }
}

/// Introduces a data block containing data, such as VEL, REF, etc.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct DataBlockHeader {
    data_block_type: [u8; 1],
    data_name: [u8; 3],
}

impl DataBlockHeader {
    pub fn data_block_type(&self) -> &[u8; 1] {
        &self.data_block_type
    }

    /// Data name, e.g. "REF", "VEL", etc.
    pub fn data_name(&self) -> &[u8; 3] {
        &self.data_name
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeData {
    data_block_header: DataBlockHeader,
    lrtup: u16,
    version_major: u8,
    version_minor: u8,
    lat: f32,
    long: f32,
    site_height: u16,
    feedhorn_height: u16,
    calibration_constant: f32,
    shvtx_power_hor: f32,
    shvtx_power_ver: f32,
    system_differential_reflectivity: f32,
    initial_system_differential_phase: f32,
    volume_coverage_pattern_number: u16,
    processing_status: u16,
}

impl VolumeData {
    pub fn data_block_header(&self) -> &DataBlockHeader {
        &self.data_block_header
    }

    pub fn lrtup(&self) -> u16 {
        self.lrtup
    }

    pub fn version_major(&self) -> u8 {
        self.version_major
    }

    pub fn version_minor(&self) -> u8 {
        self.version_minor
    }

    pub fn lat(&self) -> f32 {
        self.lat
    }

    pub fn long(&self) -> f32 {
        self.long
    }

    pub fn site_height(&self) -> u16 {
        self.site_height
    }

    pub fn feedhorn_height(&self) -> u16 {
        self.feedhorn_height
    }

    pub fn calibration_constant(&self) -> f32 {
        self.calibration_constant
    }

    pub fn shvtx_power_hor(&self) -> f32 {
        self.shvtx_power_hor
    }

    pub fn shvtx_power_ver(&self) -> f32 {
        self.shvtx_power_ver
    }

    pub fn system_differential_reflectivity(&self) -> f32 {
        self.system_differential_reflectivity
    }

    pub fn initial_system_differential_phase(&self) -> f32 {
        self.initial_system_differential_phase
    }

    pub fn volume_coverage_pattern_number(&self) -> u16 {
        self.volume_coverage_pattern_number
    }

    pub fn processing_status(&self) -> u16 {
        self.processing_status
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ElevationData {
    data_block_header: DataBlockHeader,
    lrtup: u16,
    atmos: [u8; 2],
    calib_const: f32,
}

impl ElevationData {
    pub fn data_block_header(&self) -> &DataBlockHeader {
        &self.data_block_header
    }

    /// Size of data block in bytes
    pub fn lrtup(&self) -> u16 {
        self.lrtup
    }

    /// Atmospheric Attenuation Factor
    pub fn atmos(&self) -> &[u8; 2] {
        &self.atmos
    }

    /// Scaling constant used by the Signal Processor for this elevation to calculate reflectivity
    pub fn calib_const(&self) -> f32 {
        self.calib_const
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct RadialData {
    data_block_header: DataBlockHeader,
    lrtup: u16,
    unambiguous_range: u16,
    noise_level_horz: f32,
    noise_level_vert: f32,
    nyquist_velocity: u16,
    radial_flags: u16,
    calib_const_horz_chan: f32,
    calib_const_vert_chan: f32,
}

impl RadialData {
    pub fn data_block_header(&self) -> &DataBlockHeader {
        &self.data_block_header
    }

    /// Size of data block in bytes
    pub fn lrtup(&self) -> u16 {
        self.lrtup
    }

    /// Unambiguous Range, Interval Size
    pub fn unambiguous_range(&self) -> u16 {
        self.unambiguous_range
    }

    pub fn noise_level_horz(&self) -> f32 {
        self.noise_level_horz
    }

    pub fn noise_level_vert(&self) -> f32 {
        self.noise_level_vert
    }

    pub fn nyquist_velocity(&self) -> u16 {
        self.nyquist_velocity
    }

    pub fn radial_flags(&self) -> u16 {
        self.radial_flags
    }

    pub fn calib_const_horz_chan(&self) -> f32 {
        self.calib_const_horz_chan
    }

    pub fn calib_const_vert_chan(&self) -> f32 {
        self.calib_const_vert_chan
    }
}

pub struct DataMoment {
    data: GenericData,
    moment_data: Vec<u8>,
}

impl DataMoment {
    pub(crate) fn new(data: GenericData, moment_data: Vec<u8>) -> Self {
        Self { data, moment_data }
    }

    pub fn data(&self) -> &GenericData {
        &self.data
    }

    pub fn moment_data(&self) -> &[u8] {
        &self.moment_data
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct GenericData {
    data_block_type: [u8; 1],
    data_name: [u8; 3],
    reserved: u32,
    number_data_moment_gates: u16,
    data_moment_range: u16,
    data_moment_range_sample_interval: u16,
    tover: u16,
    snr_threshold: u16,
    control_flags: u8,
    data_word_size: u8,
    scale: f32,
    offset: f32,
}

impl GenericData {
    pub fn data_block_type(&self) -> &[u8; 1] {
        &self.data_block_type
    }

    pub fn data_name(&self) -> &[u8; 3] {
        &self.data_name
    }

    pub fn reserved(&self) -> u32 {
        self.reserved
    }

    /// Number of data moment gates for current radial
    pub fn number_data_moment_gates(&self) -> u16 {
        self.number_data_moment_gates
    }

    /// Range to center of first range gate
    pub fn data_moment_range(&self) -> u16 {
        self.data_moment_range
    }

    /// Size of data moment sample interval
    pub fn data_moment_range_sample_interval(&self) -> u16 {
        self.data_moment_range_sample_interval
    }

    /// Threshold parameter which specifies the minimum difference in echo power between two
    /// resolution gates for them not to be labeled "overlayed"
    pub fn tover(&self) -> u16 {
        self.tover
    }

    /// SNR threshold for valid data
    pub fn snr_threshold(&self) -> u16 {
        self.snr_threshold
    }

    /// Indicates special control features
    pub fn control_flags(&self) -> u8 {
        self.control_flags
    }

    /// Number of bits (DWS) used for storing data for each Data Moment gate
    pub fn data_word_size(&self) -> u8 {
        self.data_word_size
    }

    /// Scale value used to convert Data Moments from integer to floating point data
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Offset value used to convert Data Moments from integer to floating point data
    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn moment_size(&self) -> usize {
        self.number_data_moment_gates as usize * self.data_word_size as usize / 8
    }
}
