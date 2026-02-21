use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::NaiveDate;
use nexrad_data::aws::archive::Identifier;
use nexrad_data::volume::{self, Record};
use nexrad_decode::messages::{MessageHeader, MessageType};
use tokio::task::JoinHandle;
use zerocopy::FromBytes;

use crate::ui::text_input::TextInput;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Application mode (high-level state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Menu,
    LocalBrowser,
    AwsBrowser,
    Loading,
    Inspector,
}

/// Current view in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    File,
    Record,
    Message,
}

/// Filesystem entry type
#[derive(Debug, Clone)]
pub enum FsEntry {
    ParentDir,
    Directory(PathBuf),
    File(PathBuf),
}

/// Local filesystem browser state
pub struct LocalBrowserState {
    pub current_dir: PathBuf,
    pub entries: Vec<FsEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

/// AWS browser step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwsStep {
    EnterSite,
    EnterDate,
    SelectFile,
}

/// AWS archive browser state
pub struct AwsBrowserState {
    pub step: AwsStep,
    pub site_input: TextInput,
    pub date_input: TextInput,
    pub files: Vec<Identifier>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl AwsBrowserState {
    pub fn new() -> Self {
        Self {
            step: AwsStep::EnterSite,
            site_input: TextInput::new("Radar Site", "e.g., KDMX"),
            date_input: TextInput::new("Date (YYYY-MM-DD)", "e.g., 2024-01-15"),
            files: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
        }
    }
}

/// Pending async operation
pub enum PendingOperation {
    ListFiles(JoinHandle<nexrad_data::result::Result<Vec<Identifier>>>),
    DownloadFile(JoinHandle<nexrad_data::result::Result<volume::File>>),
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
    /// Logical index of this message in the record
    pub index: usize,
    /// Byte offset in decompressed record
    pub offset: usize,
    /// Total message size in bytes
    pub size: usize,
    /// Full message binary data
    pub data: Vec<u8>,
    /// Number of segments this message spans (1 for non-segmented)
    pub segment_count: usize,
    /// Raw message indices before segment combining (for display as "253-257")
    pub raw_indices: Vec<usize>,
}

/// Summary of a record's contents
#[derive(Debug, Clone)]
pub struct RecordSummary {
    pub record_type: String,
    pub products: String,
    pub elevation: String,
    pub azimuth: String,
}

/// Application state
pub struct App {
    /// Application mode
    pub mode: AppMode,

    /// Menu selection index
    pub menu_selected: usize,

    /// Local browser state
    pub local_browser: Option<LocalBrowserState>,

    /// AWS browser state
    pub aws_browser: Option<AwsBrowserState>,

    /// Pending async operation
    pub pending_operation: Option<PendingOperation>,

    /// Loading message
    pub loading_message: String,

    /// Spinner animation frame
    pub spinner_frame: usize,

    /// Error message
    pub error: Option<String>,

    /// Whether to quit the application
    pub should_quit: bool,

    /// Path to the volume file (optional when no file loaded)
    pub file_path: Option<PathBuf>,

    /// Volume file header (optional when no file loaded)
    pub header: Option<volume::Header>,

    /// Raw file data (optional when no file loaded)
    file_data: Option<Vec<u8>>,

    /// List of record information
    pub records: Vec<RecordInfo>,

    /// Cache of decompressed records (record index -> decompressed data)
    decompressed_cache: HashMap<usize, Vec<u8>>,

    /// Cache of decoded messages per record (record index -> messages)
    messages_cache: HashMap<usize, Vec<MessageInfo>>,

    /// Current view (when in Inspector mode)
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

    /// When the status message was set (for auto-dismiss)
    status_message_time: Option<Instant>,

    /// Whether to filter out empty/unknown messages in record view
    pub filter_empty_unknown: bool,
}

impl App {
    /// Create a new app in interactive mode (no file loaded)
    pub fn new_interactive() -> Self {
        Self {
            mode: AppMode::Menu,
            menu_selected: 0,
            local_browser: None,
            aws_browser: None,
            pending_operation: None,
            loading_message: String::new(),
            spinner_frame: 0,
            error: None,
            should_quit: false,
            file_path: None,
            header: None,
            file_data: None,
            records: Vec::new(),
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
            status_message_time: None,
            filter_empty_unknown: true,
        }
    }

    /// Create a new app with a file loaded (CLI mode)
    pub fn new_with_file(file_path: &Path) -> AppResult<Self> {
        let mut app = Self::new_interactive();
        app.load_local_file(file_path)?;
        app.mode = AppMode::Inspector;
        Ok(app)
    }

    /// Get the volume file (recreated from cached data)
    fn volume_file(&self) -> Option<volume::File> {
        self.file_data
            .as_ref()
            .map(|data| volume::File::new(data.clone()))
    }

    /// Load a local file and populate the inspector state
    pub fn load_local_file(&mut self, path: &Path) -> AppResult<()> {
        let mut file = File::open(path)?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)?;

        let mut volume_file = volume::File::new(file_data.clone());
        if volume_file.compressed() {
            volume_file = volume_file.decompress()?;
            file_data = volume_file.data().to_vec();
        }
        let header = volume_file
            .header()
            .cloned()
            .ok_or("Failed to parse volume header")?;

        let records: Vec<RecordInfo> = volume_file
            .records()?
            .into_iter()
            .enumerate()
            .map(|(index, record)| RecordInfo {
                index,
                compressed: record.compressed(),
                size: record.data().len(),
            })
            .collect();

        self.file_path = Some(path.to_path_buf());
        self.header = Some(header);
        self.file_data = Some(file_data);
        self.records = records;
        self.decompressed_cache.clear();
        self.messages_cache.clear();
        self.view = View::File;
        self.selected_record = 0;
        self.selected_message = 0;

        Ok(())
    }

    /// Load a volume file from AWS download
    pub fn load_aws_file(
        &mut self,
        identifier: &Identifier,
        volume_file: volume::File,
    ) -> AppResult<()> {
        let header = volume_file
            .header()
            .cloned()
            .ok_or("Failed to parse volume header")?;

        let file_data = volume_file.data().to_vec();
        let records: Vec<RecordInfo> = volume_file
            .records()?
            .into_iter()
            .enumerate()
            .map(|(index, record)| RecordInfo {
                index,
                compressed: record.compressed(),
                size: record.data().len(),
            })
            .collect();

        self.file_path = Some(PathBuf::from(identifier.name()));
        self.header = Some(header);
        self.file_data = Some(file_data);
        self.records = records;
        self.decompressed_cache.clear();
        self.messages_cache.clear();
        self.view = View::File;
        self.selected_record = 0;
        self.selected_message = 0;

        Ok(())
    }

    /// Return to the main menu
    pub fn return_to_menu(&mut self) {
        self.mode = AppMode::Menu;
        self.file_path = None;
        self.header = None;
        self.file_data = None;
        self.records.clear();
        self.decompressed_cache.clear();
        self.messages_cache.clear();
    }

    /// Initialize local browser
    pub fn init_local_browser(&mut self) {
        // Start in ./downloads if it exists, otherwise use current directory
        let downloads_dir = PathBuf::from("./downloads");
        let current_dir = if downloads_dir.is_dir() {
            downloads_dir
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        let entries = Self::read_directory(&current_dir);

        self.local_browser = Some(LocalBrowserState {
            current_dir,
            entries,
            selected_index: 0,
            scroll_offset: 0,
        });
        self.mode = AppMode::LocalBrowser;
    }

    /// Initialize AWS browser
    pub fn init_aws_browser(&mut self) {
        self.aws_browser = Some(AwsBrowserState::new());
        self.mode = AppMode::AwsBrowser;
    }

    /// Read a directory and return sorted entries
    fn read_directory(path: &Path) -> Vec<FsEntry> {
        let mut entries = Vec::new();

        if path.parent().is_some() {
            entries.push(FsEntry::ParentDir);
        }

        if let Ok(read_dir) = std::fs::read_dir(path) {
            let mut items: Vec<_> = read_dir
                .filter_map(|e| e.ok())
                .map(|entry| {
                    let path = entry.path();
                    if path.is_dir() {
                        FsEntry::Directory(path)
                    } else {
                        FsEntry::File(path)
                    }
                })
                .collect();

            items.sort_by(|a, b| match (a, b) {
                (FsEntry::Directory(a), FsEntry::Directory(b)) => a.cmp(b),
                (FsEntry::File(a), FsEntry::File(b)) => a.cmp(b),
                (FsEntry::Directory(_), FsEntry::File(_)) => std::cmp::Ordering::Less,
                (FsEntry::File(_), FsEntry::Directory(_)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            });

            entries.extend(items);
        }

        entries
    }

    /// Handle local browser navigation
    pub fn local_browser_enter(&mut self) -> AppResult<()> {
        if let Some(ref mut state) = self.local_browser {
            if let Some(entry) = state.entries.get(state.selected_index).cloned() {
                match entry {
                    FsEntry::ParentDir => {
                        if let Some(parent) = state.current_dir.parent() {
                            state.current_dir = parent.to_path_buf();
                            state.entries = Self::read_directory(&state.current_dir);
                            state.selected_index = 0;
                            state.scroll_offset = 0;
                        }
                    }
                    FsEntry::Directory(path) => {
                        state.current_dir = path;
                        state.entries = Self::read_directory(&state.current_dir);
                        state.selected_index = 0;
                        state.scroll_offset = 0;
                    }
                    FsEntry::File(path) => match self.load_local_file(&path) {
                        Ok(()) => {
                            self.mode = AppMode::Inspector;
                        }
                        Err(e) => {
                            self.error = Some(format!("Failed to load file: {}", e));
                        }
                    },
                }
            }
        }
        Ok(())
    }

    /// Start listing AWS files
    pub fn start_aws_list(&mut self, site: String, date: NaiveDate) {
        self.mode = AppMode::Loading;
        self.loading_message = format!("Listing files for {} on {}...", site, date);

        let handle =
            tokio::spawn(async move { nexrad_data::aws::archive::list_files(&site, &date).await });

        self.pending_operation = Some(PendingOperation::ListFiles(handle));
    }

    /// Try to load a cached AWS file from ./downloads
    /// Returns true if the file was found and loaded successfully
    pub fn try_load_cached_aws_file(&mut self, identifier: &Identifier) -> bool {
        let downloads_dir = PathBuf::from("./downloads");
        let file_path = downloads_dir.join(identifier.name());

        if file_path.exists() {
            match self.load_local_file(&file_path) {
                Ok(()) => {
                    self.mode = AppMode::Inspector;
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Start downloading an AWS file
    pub fn start_aws_download(&mut self, identifier: Identifier) {
        self.mode = AppMode::Loading;
        self.loading_message = format!("Downloading {}...", identifier.name());

        let filename = identifier.name().to_string();
        let handle = tokio::spawn(async move {
            let result = nexrad_data::aws::archive::download_file(identifier).await;

            // Save to ./downloads if successful
            if let Ok(ref volume_file) = result {
                let downloads_dir = PathBuf::from("./downloads");
                if !downloads_dir.exists() {
                    let _ = std::fs::create_dir(&downloads_dir);
                }

                let file_path = downloads_dir.join(&filename);
                if let Ok(mut file) = std::fs::File::create(&file_path) {
                    let _ = file.write_all(volume_file.data());
                }
            }

            result
        });

        self.pending_operation = Some(PendingOperation::DownloadFile(handle));
    }

    /// Poll pending async operations
    pub async fn poll_pending_operations(&mut self) -> AppResult<()> {
        if let Some(ref mut op) = self.pending_operation {
            let is_finished = match op {
                PendingOperation::ListFiles(handle) => handle.is_finished(),
                PendingOperation::DownloadFile(handle) => handle.is_finished(),
            };

            if is_finished {
                let op = self.pending_operation.take().unwrap();
                match op {
                    PendingOperation::ListFiles(handle) => match handle.await {
                        Ok(Ok(files)) => {
                            if let Some(ref mut aws) = self.aws_browser {
                                aws.files = files;
                                aws.step = AwsStep::SelectFile;
                                aws.selected_index = 0;
                                aws.scroll_offset = 0;
                            }
                            self.mode = AppMode::AwsBrowser;
                        }
                        Ok(Err(e)) => {
                            self.error = Some(format!("Failed to list files: {}", e));
                            self.mode = AppMode::AwsBrowser;
                        }
                        Err(e) => {
                            self.error = Some(format!("Task error: {}", e));
                            self.mode = AppMode::AwsBrowser;
                        }
                    },
                    PendingOperation::DownloadFile(handle) => match handle.await {
                        Ok(Ok(volume_file)) => {
                            let identifier_opt = self
                                .aws_browser
                                .as_ref()
                                .and_then(|aws| aws.files.get(aws.selected_index).cloned());

                            if let Some(identifier) = identifier_opt {
                                match self.load_aws_file(&identifier, volume_file) {
                                    Ok(()) => {
                                        self.mode = AppMode::Inspector;
                                    }
                                    Err(e) => {
                                        self.error = Some(format!("Failed to load file: {}", e));
                                        self.mode = AppMode::AwsBrowser;
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            self.error = Some(format!("Failed to download file: {}", e));
                            self.mode = AppMode::AwsBrowser;
                        }
                        Err(e) => {
                            self.error = Some(format!("Task error: {}", e));
                            self.mode = AppMode::AwsBrowser;
                        }
                    },
                }
            }
        }
        Ok(())
    }

    /// Update spinner animation
    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 8;
    }

    /// Dismiss error overlay
    pub fn dismiss_error(&mut self) {
        self.error = None;
    }

    /// Set a status message (auto-dismisses after a few seconds)
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_message_time = Some(Instant::now());
    }

    /// Clear expired status message (call this in the main loop)
    pub fn tick_status_message(&mut self) {
        const STATUS_DURATION_SECS: u64 = 3;
        if let Some(time) = self.status_message_time {
            if time.elapsed().as_secs() >= STATUS_DURATION_SECS {
                self.status_message = None;
                self.status_message_time = None;
            }
        }
    }

    /// Get or decompress a record
    pub fn get_decompressed_record(&mut self, index: usize) -> AppResult<&[u8]> {
        if !self.decompressed_cache.contains_key(&index) {
            let volume_file = self.volume_file().ok_or("No file loaded")?;
            let records = volume_file.records()?;
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

            // Track raw message index as we iterate (segments are combined into single messages)
            let mut raw_index = 0usize;
            let message_infos: Vec<MessageInfo> = messages
                .iter()
                .enumerate()
                .map(|(index, msg)| {
                    let offset = msg.offset();
                    let size = msg.size();
                    let segment_count = msg.headers().count();

                    // Generate raw indices for this message (before segment combining)
                    let raw_indices: Vec<usize> = (raw_index..raw_index + segment_count).collect();
                    raw_index += segment_count;

                    MessageInfo {
                        index,
                        offset,
                        size,
                        data: data[offset..offset + size].to_vec(),
                        segment_count,
                        raw_indices,
                    }
                })
                .collect();

            self.messages_cache.insert(record_index, message_infos);
        }

        Ok(self.messages_cache.get(&record_index).unwrap())
    }

    /// Check if a message should be filtered out (empty or unknown)
    fn should_filter_message(msg: &MessageInfo) -> bool {
        match Self::get_message_header(&msg.data) {
            Some(hdr) => matches!(hdr.message_type(), MessageType::Unknown(_)),
            None => true, // No valid header = empty/unparseable
        }
    }

    /// Get messages for display, respecting the filter setting
    pub fn get_displayed_messages(&mut self, record_index: usize) -> AppResult<Vec<MessageInfo>> {
        let filter = self.filter_empty_unknown;
        let messages = self.get_messages(record_index)?;
        if filter {
            Ok(messages
                .iter()
                .filter(|m| !Self::should_filter_message(m))
                .cloned()
                .collect())
        } else {
            Ok(messages.to_vec())
        }
    }

    /// Get the currently selected message data (cloned to avoid borrow issues)
    pub fn current_message_data(&mut self) -> AppResult<Vec<u8>> {
        let record_index = self.selected_record;
        let message_index = self.selected_message;
        let messages = self.get_displayed_messages(record_index)?;
        messages
            .get(message_index)
            .map(|m| m.data.clone())
            .ok_or_else(|| "Message not found".into())
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
                let len = self.records.len();
                self.selected_record = if self.selected_record > 0 {
                    self.selected_record - 1
                } else if len > 0 {
                    len - 1
                } else {
                    0
                };
            }
            View::Record => {
                let record_index = self.selected_record;
                if let Ok(messages) = self.get_displayed_messages(record_index) {
                    let msg_count = messages.len();
                    self.selected_message = if self.selected_message > 0 {
                        self.selected_message - 1
                    } else if msg_count > 0 {
                        msg_count - 1
                    } else {
                        0
                    };
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
                let len = self.records.len();
                self.selected_record = if len > 0 && self.selected_record < len - 1 {
                    self.selected_record + 1
                } else {
                    0
                };
            }
            View::Record => {
                let record_index = self.selected_record;
                if let Ok(messages) = self.get_displayed_messages(record_index) {
                    let msg_count = messages.len();
                    self.selected_message =
                        if msg_count > 0 && self.selected_message < msg_count - 1 {
                            self.selected_message + 1
                        } else {
                            0
                        };
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
        match self.mode {
            AppMode::Menu => {
                // At top level menu
            }
            AppMode::LocalBrowser | AppMode::AwsBrowser => {
                self.mode = AppMode::Menu;
            }
            AppMode::Loading => {
                // Could implement cancellation here
            }
            AppMode::Inspector => {
                match self.view {
                    View::File => {
                        // Return to menu from inspector
                        self.return_to_menu();
                    }
                    View::Record => {
                        self.view = View::File;
                    }
                    View::Message => {
                        self.view = View::Record;
                    }
                }
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

    /// Toggle filtering of empty/unknown messages
    pub fn toggle_filter(&mut self) {
        self.filter_empty_unknown = !self.filter_empty_unknown;
        self.selected_message = 0;
    }

    /// Decompress the currently selected record without entering it
    pub fn decompress_selected(&mut self) {
        if self.view == View::File {
            let index = self.selected_record;
            if let Some(record) = self.records.get(index) {
                if record.compressed && !self.decompressed_cache.contains_key(&index) {
                    match self.get_decompressed_record(index) {
                        Ok(_) => {
                            // Also parse messages so summary can be generated
                            let _ = self.get_messages(index);
                            self.set_status_message(format!("Decompressed record {}", index));
                        }
                        Err(e) => {
                            self.set_status_message(format!("Failed to decompress: {}", e));
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

    /// Bulk decompress all records
    pub fn decompress_all_records(&mut self) {
        if self.view != View::File {
            return;
        }

        let total_records = self.records.len();
        let mut decompressed_count = 0;
        let mut already_decompressed = 0;
        let mut error_count = 0;

        for index in 0..total_records {
            if let Some(record) = self.records.get(index) {
                if self.decompressed_cache.contains_key(&index) {
                    already_decompressed += 1;
                } else if record.compressed {
                    match self.get_decompressed_record(index) {
                        Ok(_) => {
                            // Also parse messages so summary can be generated
                            let _ = self.get_messages(index);
                            decompressed_count += 1;
                        }
                        Err(_) => {
                            error_count += 1;
                        }
                    }
                }
            }
        }

        let mut status_parts = Vec::new();
        if decompressed_count > 0 {
            status_parts.push(format!("Decompressed {} records", decompressed_count));
        }
        if already_decompressed > 0 {
            status_parts.push(format!("{} already decompressed", already_decompressed));
        }
        if error_count > 0 {
            status_parts.push(format!("{} errors", error_count));
        }

        self.set_status_message(status_parts.join(", "));
    }

    /// Save current record to file
    pub fn save_record(&mut self) -> AppResult<()> {
        if !matches!(self.view, View::File | View::Record) {
            self.set_status_message("Must be in file or record view to save record".to_string());
            return Ok(());
        }

        let record_index = self.selected_record;

        // Check if we have decompressed data in cache
        let (data, is_decompressed) =
            if let Some(decompressed) = self.decompressed_cache.get(&record_index) {
                // Use decompressed data from cache
                (decompressed.clone(), true)
            } else {
                // Use original compressed data from file
                let volume_file = self.volume_file().ok_or("No file loaded")?;
                let records = volume_file.records()?;
                let record = records.get(record_index).ok_or("Record not found")?;
                let is_compressed = record.compressed();
                (record.data().to_vec(), !is_compressed)
            };

        let suffix = if is_decompressed {
            "_decompressed"
        } else {
            "_compressed"
        };
        let filename = format!("record_{}{}.bin", record_index, suffix);

        let mut file = File::create(&filename)?;
        file.write_all(&data)?;

        self.set_status_message(format!("Saved to {}", filename));
        Ok(())
    }

    /// Save current message to file
    pub fn save_message(&mut self) -> AppResult<()> {
        if self.view != View::Message {
            self.set_status_message("Must be in message view to save".to_string());
            return Ok(());
        }

        let data = self.current_message_data()?;
        let filename = format!(
            "message_r{}_m{}.bin",
            self.selected_record, self.selected_message
        );

        let mut file = File::create(&filename)?;
        file.write_all(&data)?;

        self.set_status_message(format!("Saved to {}", filename));
        Ok(())
    }

    /// Get a summary of record contents (if decompressed)
    pub fn get_record_summary(&self, record_index: usize) -> Option<RecordSummary> {
        // Only generate summary if record is already decompressed
        let messages = self.messages_cache.get(&record_index)?;

        if messages.is_empty() {
            return Some(RecordSummary {
                record_type: "-".to_string(),
                products: "-".to_string(),
                elevation: "-".to_string(),
                azimuth: "-".to_string(),
            });
        }

        // Categorize messages by type
        let mut radar_data_count = 0;
        let mut metadata_types = HashSet::new();

        for msg in messages {
            if let Some(header) = Self::get_message_header(&msg.data) {
                let msg_type = header.message_type();
                match msg_type {
                    MessageType::RDADigitalRadarData
                    | MessageType::RDADigitalRadarDataGenericFormat => {
                        radar_data_count += 1;
                    }
                    _ => {
                        metadata_types.insert(msg_type);
                    }
                }
            }
        }

        let has_radar_data = radar_data_count > 0;
        let has_metadata = !metadata_types.is_empty();

        if has_radar_data && !has_metadata {
            // Pure radar data record
            Some(Self::summarize_radar_data_record(messages))
        } else if !has_radar_data && has_metadata {
            // Pure metadata record
            let types_str = Self::format_message_types(&metadata_types);
            Some(RecordSummary {
                record_type: "Metadata".to_string(),
                products: types_str,
                elevation: "-".to_string(),
                azimuth: "-".to_string(),
            })
        } else if has_radar_data && has_metadata {
            // Mixed record
            let types_str = Self::format_message_types(&metadata_types);
            Some(RecordSummary {
                record_type: "Mixed".to_string(),
                products: types_str,
                elevation: "-".to_string(),
                azimuth: "-".to_string(),
            })
        } else {
            Some(RecordSummary {
                record_type: "-".to_string(),
                products: "-".to_string(),
                elevation: "-".to_string(),
                azimuth: "-".to_string(),
            })
        }
    }

    /// Summarize a radar data record
    fn summarize_radar_data_record(messages: &[MessageInfo]) -> RecordSummary {
        use nexrad_decode::messages::{decode_messages, MessageContents};

        let mut products = HashSet::new();
        let mut min_azimuth = f32::MAX;
        let mut max_azimuth = f32::MIN;
        let mut elevation = None;

        for msg in messages {
            if let Some(header) = Self::get_message_header(&msg.data) {
                let msg_type = header.message_type();
                if matches!(
                    msg_type,
                    MessageType::RDADigitalRadarData
                        | MessageType::RDADigitalRadarDataGenericFormat
                ) {
                    // Parse the message to get detailed radar data
                    if let Ok(parsed_messages) = decode_messages(&msg.data) {
                        if let Some(parsed_msg) = parsed_messages.first() {
                            if let MessageContents::DigitalRadarData(digital_radar_data) =
                                parsed_msg.contents()
                            {
                                // Track which products are present
                                if digital_radar_data.reflectivity_data_block().is_some() {
                                    products.insert("REF");
                                }
                                if digital_radar_data.velocity_data_block().is_some() {
                                    products.insert("VEL");
                                }
                                if digital_radar_data.spectrum_width_data_block().is_some() {
                                    products.insert("SW");
                                }
                                if digital_radar_data
                                    .differential_reflectivity_data_block()
                                    .is_some()
                                {
                                    products.insert("ZDR");
                                }
                                if digital_radar_data.differential_phase_data_block().is_some() {
                                    products.insert("PHI");
                                }
                                if digital_radar_data
                                    .correlation_coefficient_data_block()
                                    .is_some()
                                {
                                    products.insert("RHO");
                                }
                                if digital_radar_data
                                    .clutter_filter_power_data_block()
                                    .is_some()
                                {
                                    products.insert("CFP");
                                }

                                // Track azimuth range
                                let az = digital_radar_data.header().azimuth_angle_raw();
                                min_azimuth = min_azimuth.min(az);
                                max_azimuth = max_azimuth.max(az);

                                // Track elevation
                                if elevation.is_none() {
                                    elevation =
                                        Some(digital_radar_data.header().elevation_angle_raw());
                                }
                            }
                        }
                    }
                }
            }
        }

        let products_str = if products.is_empty() {
            "-".to_string()
        } else {
            let mut product_list: Vec<_> = products.into_iter().collect();
            product_list.sort();
            product_list.join(",")
        };

        let azimuth_str = if min_azimuth != f32::MAX && max_azimuth != f32::MIN {
            format!("{:.1}-{:.1}°", min_azimuth, max_azimuth)
        } else {
            "-".to_string()
        };

        let elevation_str = if let Some(elev) = elevation {
            format!("{:.1}°", elev)
        } else {
            "-".to_string()
        };

        RecordSummary {
            record_type: "Radar data".to_string(),
            products: products_str,
            elevation: elevation_str,
            azimuth: azimuth_str,
        }
    }

    /// Format a set of message types as a comma-separated string
    fn format_message_types(types: &HashSet<MessageType>) -> String {
        let mut type_list: Vec<_> = types
            .iter()
            .map(|t| match t {
                MessageType::RDAStatusData => "Status",
                MessageType::RDAPerformanceMaintenanceData => "Perf/Maint",
                MessageType::RDAConsoleMessage => "RDA Console",
                MessageType::RDAVolumeCoveragePattern => "RDA VCP",
                MessageType::RDAControlCommands => "Control",
                MessageType::RPGVolumeCoveragePattern => "RPG VCP",
                MessageType::RPGClutterCensorZones => "Clutter Zones",
                MessageType::RPGRequestForData => "Request",
                MessageType::RPGConsoleMessage => "RPG Console",
                MessageType::RDALoopBackTest => "RDA Loopback",
                MessageType::RPGLoopBackTest => "RPG Loopback",
                MessageType::RDAClutterFilterBypassMap => "Clutter Bypass",
                MessageType::RDAClutterFilterMap => "Clutter Map",
                MessageType::RDAAdaptationData => "Adaptation",
                MessageType::RDAPRFData => "PRF",
                MessageType::RDALogData => "Log",
                _ => "Other",
            })
            .collect();
        type_list.sort();
        type_list.dedup();
        type_list.join(", ")
    }
}
