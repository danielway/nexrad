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

    /// All messages in sequence, with related messages grouped together
    pub message_groups: Vec<MessageGroupSummary>,

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
            write!(f, ", End: {}", end.format("%Y-%m-%d %H:%M:%S%.3f UTC"))?;
            if let Some(start) = self.earliest_collection_time {
                let duration = end.signed_duration_since(start);
                write!(f, " ({:.2}m)", duration.num_milliseconds() as f64 / 60000.0)?;
            }
        } else {
            write!(f, ", End: unknown")?;
        }
        writeln!(f)?;

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

        // Messages
        writeln!(f, "Messages:")?;
        for group in self.message_groups.iter() {
            let prefix = if group.message_type == MessageType::RDADigitalRadarDataGenericFormat {
                let msg_range = if group.start_message_index == group.end_message_index {
                    format!("  Msg {}", group.start_message_index + 1)
                } else {
                    format!(
                        "  Msg {}-{}",
                        group.start_message_index + 1,
                        group.end_message_index + 1
                    )
                };

                if group.is_continued {
                    format!("{} (cont.)", msg_range)
                } else {
                    msg_range
                }
            } else if group.start_message_index == group.end_message_index {
                format!("  Msg {}", group.start_message_index + 1)
            } else {
                format!(
                    "  Msg {}-{}",
                    group.start_message_index + 1,
                    group.end_message_index + 1
                )
            };
            writeln!(f, "{}: {}", prefix, group)?;
        }

        Ok(())
    }
}

/// Summary of a single message or group of related messages
#[derive(Clone, PartialEq, Debug)]
pub struct MessageGroupSummary {
    pub message_type: MessageType,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub message_count: usize,

    // For DigitalRadarData messages
    pub elevation_number: Option<u8>,
    pub elevation_angle: Option<f32>,
    pub start_azimuth: Option<f32>,
    pub end_azimuth: Option<f32>,
    pub data_types: Option<HashMap<String, usize>>,

    // Indicates if this group continues from a previous group
    pub is_continued: bool,

    // Absolute message indices
    pub start_message_index: usize,
    pub end_message_index: usize,
}

impl Display for MessageGroupSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.message_type == MessageType::RDADigitalRadarDataGenericFormat {
            write!(f, "Elevation: #{}", self.elevation_number.unwrap_or(0))?;

            if let Some(elev_angle) = self.elevation_angle {
                write!(f, " ({:.2}°)", elev_angle)?;
            }

            write!(
                f,
                ", Azimuth: {:.1}° to {:.1}°",
                self.start_azimuth.unwrap_or(0.0),
                self.end_azimuth.unwrap_or(0.0)
            )?;

            if let Some(start) = self.start_time {
                write!(f, ", Time: {}", start.format("%H:%M:%S%.3f"))?;

                if let Some(end) = self.end_time {
                    if start != end {
                        write!(f, " to {}", end.format("%H:%M:%S%.3f"))?;
                        let duration = end.signed_duration_since(start);
                        write!(f, " ({:.2}s)", duration.num_milliseconds() as f64 / 1000.0)?;
                    }
                }
            }

            if let Some(data_types) = &self.data_types {
                if !data_types.is_empty() {
                    writeln!(f)?;
                    write!(f, "    Data types: ")?;
                    let data_types: Vec<_> = data_types.iter().collect();

                    for (i, (data_type, count)) in data_types.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        // Use abbreviated names for common data types
                        let abbr = match data_type.as_str() {
                            "Reflectivity" => "REF",
                            "Velocity" => "VEL",
                            "Spectrum Width" => "SW",
                            "Differential Reflectivity" => "ZDR",
                            "Differential Phase" => "DP",
                            "Correlation Coefficient" => "CC",
                            "Specific Differential Phase" => "KDP",
                            _ => data_type,
                        };

                        write!(f, "{} ({})", abbr, count)?;
                    }
                }
            }
        } else {
            write!(f, "{:?}", self.message_type)?;
            if self.message_count > 1 {
                write!(f, " ({})", self.message_count)?;
            }
            if let Some(start) = self.start_time {
                write!(f, ", Time: {}", start.format("%H:%M:%S%.3f"))?;
                if let Some(end) = self.end_time {
                    if start != end {
                        write!(f, " to {}", end.format("%H:%M:%S%.3f"))?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Provides a summary of the given messages.
pub fn messages(messages: &[Message]) -> MessageSummary {
    let mut summary = MessageSummary {
        volume_coverage_patterns: HashSet::new(),
        message_groups: Vec::new(),
        earliest_collection_time: None,
        latest_collection_time: None,
    };

    let mut current_group: Option<MessageGroupSummary> = None;
    for (i, message) in messages.iter().enumerate() {
        let message_type = message.header().message_type();
        let message_time = message.header().date_time();

        if let Some(time) = message_time {
            if summary.earliest_collection_time.is_none()
                || summary.earliest_collection_time > Some(time)
            {
                summary.earliest_collection_time = Some(time);
            }
            if summary.latest_collection_time.is_none()
                || summary.latest_collection_time < Some(time)
            {
                summary.latest_collection_time = Some(time);
            }
        }

        match message.contents() {
            MessageContents::DigitalRadarData(radar_data) => {
                let elevation_number = radar_data.header.elevation_number;

                let can_continue = if let Some(group) = &current_group {
                    group.message_type == MessageType::RDADigitalRadarDataGenericFormat
                        && group.elevation_number == Some(elevation_number)
                } else {
                    false
                };

                if !can_continue {
                    if let Some(group) = current_group.take() {
                        summary.message_groups.push(group);
                    }

                    current_group = Some(MessageGroupSummary {
                        message_type: MessageType::RDADigitalRadarDataGenericFormat,
                        start_time: message_time,
                        end_time: message_time,
                        message_count: 1,
                        elevation_number: Some(elevation_number),
                        elevation_angle: Some(radar_data.header.elevation_angle),
                        start_azimuth: Some(radar_data.header.azimuth_angle),
                        end_azimuth: Some(radar_data.header.azimuth_angle),
                        data_types: Some(HashMap::new()),
                        is_continued: !summary.message_groups.is_empty()
                            && summary.message_groups.iter().rev().any(|g| {
                                g.message_type == MessageType::RDADigitalRadarDataGenericFormat
                                    && g.elevation_number == Some(elevation_number)
                            }),
                        start_message_index: i,
                        end_message_index: i,
                    });
                } else if let Some(group) = &mut current_group {
                    group.end_time = message_time;
                    group.message_count += 1;
                    group.end_azimuth = Some(radar_data.header.azimuth_angle);
                    group.end_message_index = i;
                }

                if let Some(group) = &mut current_group {
                    if let Some(data_types) = group.data_types.as_mut() {
                        let mut increment_count = |data_type: &str| {
                            let count = data_types.get(data_type).unwrap_or(&0) + 1;
                            data_types.insert(data_type.to_string(), count);
                        };

                        if radar_data.reflectivity_data_block.is_some() {
                            increment_count("Reflectivity");
                        }
                        if radar_data.velocity_data_block.is_some() {
                            increment_count("Velocity");
                        }
                        if radar_data.spectrum_width_data_block.is_some() {
                            increment_count("Spectrum Width");
                        }
                        if radar_data.differential_reflectivity_data_block.is_some() {
                            increment_count("Differential Reflectivity");
                        }
                        if radar_data.differential_phase_data_block.is_some() {
                            increment_count("Differential Phase");
                        }
                        if radar_data.correlation_coefficient_data_block.is_some() {
                            increment_count("Correlation Coefficient");
                        }
                        if radar_data.specific_diff_phase_data_block.is_some() {
                            increment_count("Specific Differential Phase");
                        }
                    }
                }

                if let Some(volume_data) = &radar_data.volume_data_block {
                    summary
                        .volume_coverage_patterns
                        .insert(volume_data.volume_coverage_pattern());
                }
            }
            _ => {
                let can_combine = if let Some(group) = &current_group {
                    group.message_type == message_type
                } else {
                    false
                };

                if !can_combine {
                    if let Some(group) = current_group.take() {
                        summary.message_groups.push(group);
                    }

                    current_group = Some(MessageGroupSummary {
                        message_type,
                        start_time: message_time,
                        end_time: message_time,
                        message_count: 1,
                        elevation_number: None,
                        elevation_angle: None,
                        start_azimuth: None,
                        end_azimuth: None,
                        data_types: None,
                        is_continued: false,
                        start_message_index: i,
                        end_message_index: i,
                    });
                } else if let Some(group) = &mut current_group {
                    group.end_time = message_time;
                    group.message_count += 1;
                    group.end_message_index = i;
                }
            }
        }
    }

    if let Some(group) = current_group {
        summary.message_groups.push(group);
    }

    summary
}
