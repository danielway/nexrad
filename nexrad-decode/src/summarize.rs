use crate::messages::digital_radar_data;
use crate::messages::{Message, MessageContents, MessageType};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Summary of a set of messages.
#[derive(Clone, PartialEq, Debug)]
pub struct MessageSummary {
    /// The distinct volume coverage patterns found in these messages.
    pub volume_coverage_patterns: HashSet<digital_radar_data::VolumeCoveragePattern>,

    /// The number of messages of each type in the order they appear. Multiple messages of the same
    /// type will be grouped together if consecutive.
    pub message_types: Vec<(MessageType, usize)>,

    /// Summaries of each scan found in these messages.
    pub scans: Vec<ScanSummary>,

    pub earliest_collection_time: Option<DateTime<Utc>>,
    pub latest_collection_time: Option<DateTime<Utc>>,
}

impl Display for MessageSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Time information
        if let Some(start) = self.earliest_collection_time {
            write!(f, "Start: {}", start.format("%Y-%m-%d %H:%M:%S%.3f UTC"))?;
        } else {
            write!(f, "Start: unknown")?;
        }
        
        if let Some(end) = self.latest_collection_time {
            writeln!(f, ", End: {}", end.format("%Y-%m-%d %H:%M:%S%.3f UTC"))?;
        } else {
            writeln!(f, ", End: unknown")?;
        }
        
        // Volume coverage patterns
        write!(f, "VCPs: ")?;
        if self.volume_coverage_patterns.is_empty() {
            writeln!(f, "none")?;
        } else {
            let vcps: Vec<_> = self.volume_coverage_patterns.iter().collect();
            for (i, vcp) in vcps.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{:?}", vcp)?;
            }
            writeln!(f)?;
        }
        
        // Message types
        writeln!(f, "Message types:")?;
        for (msg_type, count) in &self.message_types {
            writeln!(f, "  {:?}: {}", msg_type, count)?;
        }
        
        // Scans
        writeln!(f, "Scans: {}", self.scans.len())?;
        for (i, scan) in self.scans.iter().enumerate() {
            writeln!(f, "  Scan {}: {}", i + 1, scan)?;
        }
        
        Ok(())
    }
}

/// Summary of a single scan.
#[derive(Clone, PartialEq, Debug)]
pub struct ScanSummary {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,

    pub elevation: u8,

    pub start_azimuth: f32,
    pub end_azimuth: f32,

    /// The number of messages containing a given radar data type.
    pub data_types: HashMap<String, usize>,
}

impl Display for ScanSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Elevation: {}°, Azimuth: {:.1}° to {:.1}°", 
               self.elevation, self.start_azimuth, self.end_azimuth)?;
        
        if let Some(start) = self.start_time {
            write!(f, ", Time: {}", start.format("%H:%M:%S%.3f"))?;
            
            if let Some(end) = self.end_time {
                if start != end {
                    write!(f, " to {}", end.format("%H:%M:%S%.3f"))?;
                }
            }
        }
        
        if !self.data_types.is_empty() {
            writeln!(f, "")?;
            write!(f, "    Data types: ")?;
            let data_types: Vec<_> = self.data_types.iter().collect();
            for (i, (data_type, count)) in data_types.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{} ({})", data_type, count)?;
            }
        }
        
        Ok(())
    }
}

/// Provides a summary of the given messages.
pub fn messages(messages: &[Message]) -> MessageSummary {
    let mut summary = MessageSummary {
        volume_coverage_patterns: HashSet::new(),
        message_types: Vec::new(),
        scans: Vec::new(),
        earliest_collection_time: None,
        latest_collection_time: None,
    };

    if let Some(first_message) = messages.first() {
        summary.earliest_collection_time = first_message.header().date_time();
    }

    let mut scan_summary = None;
    for message in messages {
        process_message(&mut summary, &mut scan_summary, message);
    }

    if let Some(scan_summary) = scan_summary.take() {
        summary.scans.push(scan_summary);
    }

    summary
}

fn process_message(
    summary: &mut MessageSummary,
    scan_summary: &mut Option<ScanSummary>,
    message: &Message,
) {
    let message_type = message.header().message_type();
    if let Some((last_message_type, count)) = summary.message_types.last_mut() {
        if *last_message_type == message_type {
            *count += 1;
        } else {
            summary.message_types.push((message_type, 1));
        }
    } else {
        summary.message_types.push((message_type, 1));
    }

    if let MessageContents::DigitalRadarData(radar_data_message) = message.contents() {
        process_digital_radar_data_message(summary, scan_summary, radar_data_message);
        return;
    }

    if let Some(scan_summary) = scan_summary.take() {
        summary.scans.push(scan_summary);
    }
}

fn process_digital_radar_data_message(
    summary: &mut MessageSummary,
    scan_summary: &mut Option<ScanSummary>,
    message: &digital_radar_data::Message,
) {
    let elevation_changed =
        |summary: &mut ScanSummary| summary.elevation != message.header.elevation_number;

    if let Some(scan_summary) = scan_summary.take_if(elevation_changed) {
        summary.scans.push(scan_summary);
    }

    let scan_summary = scan_summary.get_or_insert_with(|| ScanSummary {
        start_time: message.header.date_time(),
        end_time: message.header.date_time(),
        elevation: message.header.elevation_number,
        start_azimuth: message.header.azimuth_angle,
        end_azimuth: message.header.azimuth_angle,
        data_types: HashMap::new(),
    });

    if message.header.date_time().is_some() {
        if summary.earliest_collection_time.is_none()
            || summary.earliest_collection_time > message.header.date_time()
        {
            summary.earliest_collection_time = message.header.date_time();
        }

        if summary.latest_collection_time.is_none()
            || summary.latest_collection_time < message.header.date_time()
        {
            summary.latest_collection_time = message.header.date_time();
        }

        if scan_summary.start_time.is_none() || scan_summary.start_time > message.header.date_time()
        {
            scan_summary.start_time = message.header.date_time();
        }

        if scan_summary.end_time.is_none() || scan_summary.end_time < message.header.date_time() {
            scan_summary.end_time = message.header.date_time();
        }
    }

    if let Some(volume_data) = &message.volume_data_block {
        summary
            .volume_coverage_patterns
            .insert(volume_data.volume_coverage_pattern());
    }

    scan_summary.end_azimuth = message.header.azimuth_angle;

    let mut increment_count = |data_type: &str| {
        let count = scan_summary.data_types.get(data_type).unwrap_or(&0) + 1;
        scan_summary.data_types.insert(data_type.to_string(), count);
    };

    if message.reflectivity_data_block.is_some() {
        increment_count("Reflectivity");
    }
    if message.velocity_data_block.is_some() {
        increment_count("Velocity");
    }
    if message.spectrum_width_data_block.is_some() {
        increment_count("Spectrum Width");
    }
    if message.differential_reflectivity_data_block.is_some() {
        increment_count("Differential Reflectivity");
    }
    if message.differential_phase_data_block.is_some() {
        increment_count("Differential Phase");
    }
    if message.correlation_coefficient_data_block.is_some() {
        increment_count("Correlation Coefficient");
    }
    if message.specific_diff_phase_data_block.is_some() {
        increment_count("Specific Differential Phase");
    }
}
