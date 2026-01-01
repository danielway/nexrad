use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use nexrad_data::volume::{self, Record};
use nexrad_decode::messages::MessageHeader;
use zerocopy::FromBytes;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Current view in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    File,
    Record,
    Message,
}

/// Tab within the message view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTab {
    Hex,
    Parsed,
}

/// Information about a record in the volume file
#[derive(Debug)]
pub struct RecordInfo {
    pub index: usize,
    pub compressed: bool,
    pub size: usize,
}

/// Information about a message within a record
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub index: usize,
    pub offset: usize,
    pub size: usize,
    pub data: Vec<u8>,
}

/// Application state
pub struct App {
    /// Path to the volume file
    pub file_path: PathBuf,

    /// Volume file header
    pub header: volume::Header,

    /// Raw file data
    file_data: Vec<u8>,

    /// List of record information
    pub records: Vec<RecordInfo>,

    /// Cache of decompressed records (record index -> decompressed data)
    decompressed_cache: HashMap<usize, Vec<u8>>,

    /// Cache of decoded messages per record (record index -> messages)
    messages_cache: HashMap<usize, Vec<MessageInfo>>,

    /// Current view
    pub view: View,

    /// Current tab in message view
    pub message_tab: MessageTab,

    /// Selected index in file view (record list)
    pub selected_record: usize,

    /// Selected index in record view (message list)
    pub selected_message: usize,

    /// Scroll offset for hex view
    pub hex_scroll: usize,

    /// Scroll offset for parsed view
    pub parsed_scroll: usize,

    /// Whether help overlay is shown
    pub show_help: bool,

    /// Status message to display
    pub status_message: Option<String>,
}

impl App {
    pub fn new(file_path: &Path) -> AppResult<Self> {
        // Read the entire file
        let mut file = File::open(file_path)?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)?;

        // Parse as volume file
        let volume_file = volume::File::new(file_data.clone());
        let header = volume_file
            .header()
            .cloned()
            .ok_or("Failed to parse volume header")?;

        // Collect record information
        let records: Vec<RecordInfo> = volume_file
            .records()
            .into_iter()
            .enumerate()
            .map(|(index, record)| RecordInfo {
                index,
                compressed: record.compressed(),
                size: record.data().len(),
            })
            .collect();

        Ok(Self {
            file_path: file_path.to_path_buf(),
            header,
            file_data,
            records,
            decompressed_cache: HashMap::new(),
            messages_cache: HashMap::new(),
            view: View::File,
            message_tab: MessageTab::Hex,
            selected_record: 0,
            selected_message: 0,
            hex_scroll: 0,
            parsed_scroll: 0,
            show_help: false,
            status_message: None,
        })
    }

    /// Get the volume file (recreated from cached data)
    fn volume_file(&self) -> volume::File {
        volume::File::new(self.file_data.clone())
    }

    /// Get or decompress a record
    pub fn get_decompressed_record(&mut self, index: usize) -> AppResult<&[u8]> {
        if !self.decompressed_cache.contains_key(&index) {
            let volume_file = self.volume_file();
            let records = volume_file.records();
            let record = records.get(index).ok_or("Record not found")?;

            let decompressed = if record.compressed() {
                let decompressed_record = record.decompress()?;
                decompressed_record.data().to_vec()
            } else {
                record.data().to_vec()
            };

            self.decompressed_cache.insert(index, decompressed);
        }

        Ok(self.decompressed_cache.get(&index).unwrap())
    }

    /// Get or decode messages for a record
    pub fn get_messages(&mut self, record_index: usize) -> AppResult<&[MessageInfo]> {
        if !self.messages_cache.contains_key(&record_index) {
            // First ensure the record is decompressed
            let _ = self.get_decompressed_record(record_index)?;
            let data = self.decompressed_cache.get(&record_index).unwrap().clone();

            // Use nexrad-decode to parse messages and get offset/size info
            let record = Record::new(data.clone());
            let messages = record.messages()?;

            let message_infos: Vec<MessageInfo> = messages
                .iter()
                .enumerate()
                .map(|(index, msg)| {
                    let offset = msg.offset();
                    let size = msg.size();
                    MessageInfo {
                        index,
                        offset,
                        size,
                        data: data[offset..offset + size].to_vec(),
                    }
                })
                .collect();

            self.messages_cache.insert(record_index, message_infos);
        }

        Ok(self.messages_cache.get(&record_index).unwrap())
    }

    /// Get the currently selected message data (cloned to avoid borrow issues)
    pub fn current_message_data(&mut self) -> AppResult<Vec<u8>> {
        let record_index = self.selected_record;
        let message_index = self.selected_message;
        let messages = self.get_messages(record_index)?;
        messages
            .get(message_index)
            .map(|m| m.data.clone())
            .ok_or_else(|| "Message not found".into())
    }

    /// Get the number of messages in a record (if cached)
    pub fn message_count(&self, record_index: usize) -> Option<usize> {
        self.messages_cache.get(&record_index).map(|m| m.len())
    }

    /// Get the message header for a message
    pub fn get_message_header(data: &[u8]) -> Option<&MessageHeader> {
        if data.len() >= std::mem::size_of::<MessageHeader>() {
            MessageHeader::ref_from_prefix(data).ok().map(|(h, _)| h)
        } else {
            None
        }
    }

    /// Get the decompressed size for a record (if cached)
    pub fn get_decompressed_size(&self, record_index: usize) -> Option<usize> {
        self.decompressed_cache.get(&record_index).map(|d| d.len())
    }

    /// Navigate to previous item in current list
    pub fn previous(&mut self) {
        match self.view {
            View::File => {
                if self.selected_record > 0 {
                    self.selected_record -= 1;
                }
            }
            View::Record => {
                if self.selected_message > 0 {
                    self.selected_message -= 1;
                }
            }
            View::Message => match self.message_tab {
                MessageTab::Hex => {
                    if self.hex_scroll > 0 {
                        self.hex_scroll -= 1;
                    }
                }
                MessageTab::Parsed => {
                    if self.parsed_scroll > 0 {
                        self.parsed_scroll -= 1;
                    }
                }
            },
        }
    }

    /// Navigate to next item in current list
    pub fn next(&mut self) {
        match self.view {
            View::File => {
                if self.selected_record < self.records.len().saturating_sub(1) {
                    self.selected_record += 1;
                }
            }
            View::Record => {
                let record_index = self.selected_record;
                if let Some(msg_count) = self.message_count(record_index) {
                    if self.selected_message < msg_count.saturating_sub(1) {
                        self.selected_message += 1;
                    }
                }
            }
            View::Message => match self.message_tab {
                MessageTab::Hex => {
                    self.hex_scroll += 1;
                }
                MessageTab::Parsed => {
                    self.parsed_scroll += 1;
                }
            },
        }
    }

    /// Page up in current view
    pub fn page_up(&mut self) {
        for _ in 0..10 {
            self.previous();
        }
    }

    /// Page down in current view
    pub fn page_down(&mut self) {
        for _ in 0..10 {
            self.next();
        }
    }

    /// Enter selected item (drill down)
    pub fn enter(&mut self) {
        match self.view {
            View::File => {
                // Decompress the selected record and move to record view
                if self.get_decompressed_record(self.selected_record).is_ok() {
                    self.selected_message = 0;
                    self.view = View::Record;
                }
            }
            View::Record => {
                // Move to message view
                self.hex_scroll = 0;
                self.parsed_scroll = 0;
                self.view = View::Message;
            }
            View::Message => {
                // No deeper level
            }
        }
    }

    /// Go back to previous view
    pub fn back(&mut self) {
        match self.view {
            View::File => {
                // Already at top level
            }
            View::Record => {
                self.view = View::File;
            }
            View::Message => {
                self.view = View::Record;
            }
        }
    }

    /// Toggle between hex and parsed view in message view
    pub fn toggle_view(&mut self) {
        if self.view == View::Message {
            self.message_tab = match self.message_tab {
                MessageTab::Hex => MessageTab::Parsed,
                MessageTab::Parsed => MessageTab::Hex,
            };
        }
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Decompress the currently selected record without entering it
    pub fn decompress_selected(&mut self) {
        if self.view == View::File {
            let index = self.selected_record;
            if let Some(record) = self.records.get(index) {
                if record.compressed && !self.decompressed_cache.contains_key(&index) {
                    match self.get_decompressed_record(index) {
                        Ok(_) => {
                            self.status_message = Some(format!("Decompressed record {}", index));
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to decompress: {}", e));
                        }
                    }
                }
            }
        }
    }

    /// Check if a record is decompressed
    pub fn is_record_decompressed(&self, index: usize) -> bool {
        self.decompressed_cache.contains_key(&index)
    }

    /// Save current message to file
    pub fn save_message(&mut self) -> AppResult<()> {
        if self.view != View::Message {
            self.status_message = Some("Must be in message view to save".to_string());
            return Ok(());
        }

        let data = self.current_message_data()?;
        let filename = format!(
            "message_r{}_m{}.bin",
            self.selected_record, self.selected_message
        );

        let mut file = File::create(&filename)?;
        file.write_all(&data)?;

        self.status_message = Some(format!("Saved to {}", filename));
        Ok(())
    }
}
