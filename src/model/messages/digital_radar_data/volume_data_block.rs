use uom::si::f64::{Angle, Energy, Information, Length};
use crate::model::messages::digital_radar_data::ProcessingStatus;
use crate::model::messages::primitive_aliases::{Integer1, Integer2, Real4, SInteger2};

/// A volume data moment block.
pub struct VolumeDataBlock {
    /// Data block type, "R".
    data_block_type: u8,

    /// Data block name, e.g. "VOL".
    data_name: [u8; 3],

    /// Size of data block in bytes.
    lrtup: Integer2,

    /// Major version number.
    major_version_number: Integer1,

    /// Minor version number.
    minor_version_number: Integer1,

    /// Latitude of radar in degrees.
    latitude: Real4,

    /// Longitude of radar in degrees.
    longitude: Real4,

    /// Height of site base above sea level in meters.
    site_height: SInteger2,

    /// Height of feedhorn above ground in meters.
    feedhorn_height: Integer2,

    /// Reflectivity scaling factor without correction by ground noise scaling factors given in
    /// adaptation data message in dB.
    calibration_constant: Real4,

    /// Transmitter power for horizontal channel in kW.
    horizontal_shv_tx_power: Real4,

    /// Transmitter power for vertical channel in kW.
    vertical_shv_tx_power: Real4,

    /// Calibration of system ZDR in dB.
    system_differential_reflectivity: Real4,

    /// Initial DP for the system in degrees.
    initial_system_differential_phase: Real4,

    /// Identifies the volume coverage pattern in use.
    /// todo: Appendix C for available VCPs
    volume_coverage_pattern_number: Integer2,

    /// Processing option flags.
    ///
    /// Options:
    ///   0 = RxR noise
    ///   1 = CBT
    processing_status: Integer2,

    /// RPG weighted mean ZDR bias estimate in dB.
    zdr_bias_estimate_weighted_mean: Integer2,

    /// Spare.
    spare: [u8; 6],
}

impl VolumeDataBlock {
    /// Data block type, "R".
    pub fn data_block_type(&self) -> char {
        self.data_block_type as char
    }

    /// Data block name, e.g. "VOL".
    pub fn data_name(&self) -> String {
        String::from_utf8_lossy(&self.data_name).to_string()
    }

    /// Size of data block.
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup as f64)
    }

    /// Latitude of radar.
    pub fn latitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.latitude as f64)
    }
    
    /// Longitude of radar.
    pub fn longitude(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.longitude as f64)
    }
    
    /// Height of site base above sea level.
    pub fn site_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.site_height as f64)
    }
    
    /// Height of feedhorn above ground.
    pub fn feedhorn_height(&self) -> Length {
        Length::new::<uom::si::length::meter>(self.feedhorn_height as f64)
    }
    
    /// Transmitter power for horizontal channel.
    pub fn horizontal_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.horizontal_shv_tx_power as f64)
    }
    
    /// Transmitter power for vertical channel.
    pub fn vertical_shv_tx_power(&self) -> Energy {
        Energy::new::<uom::si::energy::kilojoule>(self.vertical_shv_tx_power as f64)
    }
    
    /// Initial DP for the system.
    pub fn initial_system_differential_phase(&self) -> Angle {
        Angle::new::<uom::si::angle::degree>(self.initial_system_differential_phase as f64)
    }
    
    // todo: vcp number
    
    /// Processing option flags.
    pub fn processing_status(&self) -> ProcessingStatus {
        match self.processing_status {
            0 => ProcessingStatus::RxRNoise,
            1 => ProcessingStatus::CBT,
            _ => panic!("Invalid processing status"),
        }
    }
}