/*
*
- Maintain raw os file descriptor to read and write logs into the swap file
- instead of maintaining direct json and make use of serde to json.. we have to maintain swap file
- like nvim in order to be ACID proof serde already derived on the Log Entry and its nested types but
- this file maintain and hold raw file descriptor push and read those files directly
- */

use std::{
    ffi::OsStr,
    fmt::Display,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use crate::LogEntry;

pub struct SwapLogFileWriter {
    pub logs_dir: PathBuf, // Immutable base log directory defined by env (e.g., abc/logs_directory/)
    pub log_state_file: Option<PathBuf>, // Active file path state (e.g., abc/logs_directory/<ticket_id>.swap)
    pub state_ticket_id: Option<String>, // Current ticket getting is_generating = true
    pub writer_handle: Option<BufWriter<File>>, // Direct file descriptor handle with 8KB buffer
}

#[derive(Debug)]
pub enum SwapLogFileWriterError {
    SwapFileAlreadyExists,
    LogDirAbsolute,
    SwapFileExpectedNotFound,
    TicketIdNotMatched,
    JsonFileExpectedNotFound,
}

impl Display for SwapLogFileWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapLogFileWriterError::SwapFileAlreadyExists => write!(
                f,
                "Active log_state_file already exists; commit or reset current swap state before deriving a new one"
            ),
            SwapLogFileWriterError::LogDirAbsolute => write!(
                f,
                "Log directory expected to be a valid directory path only, not a file path"
            ),
            SwapLogFileWriterError::SwapFileExpectedNotFound => {
                write!(
                    f,
                    "Expected active .swap file in log_state_file, but none was found"
                )
            }
            SwapLogFileWriterError::TicketIdNotMatched => {
                write!(
                    f,
                    "Expected Ticket ID to match between provided ticket_id and active swap file"
                )
            }
            SwapLogFileWriterError::JsonFileExpectedNotFound => {
                write!(f, "Expected Json File With Given Ticket Id Doesnt Found")
            }
        }
    }
}

impl std::error::Error for SwapLogFileWriterError {}

impl From<SwapLogFileWriterError> for io::Error {
    fn from(my_err: SwapLogFileWriterError) -> Self {
        match my_err {
            SwapLogFileWriterError::SwapFileAlreadyExists => {
                io::Error::new(io::ErrorKind::AlreadyExists, format!("{}", &my_err))
            }
            SwapLogFileWriterError::LogDirAbsolute => {
                io::Error::new(io::ErrorKind::NotADirectory, format!("{}", &my_err))
            }
            SwapLogFileWriterError::SwapFileExpectedNotFound => {
                io::Error::new(io::ErrorKind::NotFound, format!("{}", &my_err))
            }
            SwapLogFileWriterError::TicketIdNotMatched => {
                io::Error::new(io::ErrorKind::InvalidInput, format!("{}", &my_err))
            }
            SwapLogFileWriterError::JsonFileExpectedNotFound => {
                io::Error::new(io::ErrorKind::NotFound, format!("{}", &my_err))
            }
        }
    }
}

impl SwapLogFileWriter {
    // Called once to set up the base log directory
    pub fn new(logs_dir: impl Into<PathBuf>) -> io::Result<Self> {
        let logs_dir = logs_dir.into();
        fs::create_dir_all(&logs_dir)?;

        if !logs_dir.is_dir() {
            return Err(SwapLogFileWriterError::LogDirAbsolute.into());
        }

        Ok(Self {
            logs_dir,
            log_state_file: None,
            state_ticket_id: None,
            writer_handle: None,
        })
    }

    #[inline]
    // Initializes `log_state_file` as `<logs_dir>/<ticket_id>.swap` without mutating `logs_dir`
    pub fn derive_swap_file(&mut self, ticket_id: &str) -> io::Result<()> {
        if !self.logs_dir.is_dir() {
            return Err(SwapLogFileWriterError::LogDirAbsolute.into());
        }

        // If a state file is already active, check if it's the exact same ticket's swap file
        if let Some(active_file) = &self.log_state_file {
            let is_same_swap = active_file.extension() == Some(OsStr::new("swap"))
                && active_file.file_stem() == Some(OsStr::new(ticket_id));

            if is_same_swap {
                return Ok(());
            }
            // Previous ticket state wasn't reset cleanly
            return Err(SwapLogFileWriterError::SwapFileAlreadyExists.into());
        }

        // Construct new swap path from immutable base `logs_dir`
        let mut swap_path = self.logs_dir.join(ticket_id);
        swap_path.set_extension("swap");

        self.log_state_file = Some(swap_path);
        self.state_ticket_id = Some(ticket_id.to_string());

        Ok(())
    }

    #[inline]
    // Atomically renames `<ticket_id>.swap` -> `<ticket_id>.jsonl` on disk
    fn derive_json_file(&mut self) -> io::Result<()> {
        let state_file = self
            .log_state_file
            .as_mut()
            .ok_or(SwapLogFileWriterError::SwapFileExpectedNotFound)?;

        if state_file.extension() != Some(OsStr::new("swap")) {
            return Err(SwapLogFileWriterError::SwapFileExpectedNotFound.into());
        }

        let old_swap_file = state_file.clone();
        state_file.set_extension("jsonl");

        // Atomic POSIX rename on disk
        fs::rename(&old_swap_file, &state_file)?;

        Ok(())
    }

    // Appends serialized LogEntry into the active `.swap` file
    pub fn dump_log_entry(&mut self, log: &LogEntry, ticket_id: &str) -> io::Result<()> {
        let state_file = self
            .log_state_file
            .as_ref()
            .ok_or(SwapLogFileWriterError::SwapFileExpectedNotFound)?;

        // Ensure current state file is a `.swap` file
        if state_file.extension() != Some(OsStr::new("swap")) {
            return Err(SwapLogFileWriterError::SwapFileExpectedNotFound.into());
        }

        // Ensure ticket_id matches both the filename stem and state_ticket_id
        if state_file.file_stem() != Some(OsStr::new(ticket_id))
            || self.state_ticket_id.as_deref() != Some(ticket_id)
        {
            return Err(SwapLogFileWriterError::TicketIdNotMatched.into());
        }

        // Lazily open the OS file descriptor and wrap in 8KB BufWriter if not already open
        if self.writer_handle.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(state_file)?;

            self.writer_handle = Some(BufWriter::new(file));
        }

        // Write serialized JSON line using explicit reborrow `&mut *buffer_file`
        if let Some(buffer_file) = &mut self.writer_handle {
            serde_json::to_writer(&mut *buffer_file, log)?;
            buffer_file.write_all(b"\n")?;
            buffer_file.flush()?;
        }

        Ok(())
    }

    // Flushes buffers, fsyncs to disk, closes handle, renames `.swap` -> `.jsonl`, and resets state
    pub fn commit_turn(&mut self) -> io::Result<()> {
        if self.state_ticket_id.is_some() {
            if let Some(mut buffer_file) = self.writer_handle.take() {
                buffer_file.flush()?;
                // Force OS kernel page cache to physical disk (fsync)
                buffer_file.get_ref().sync_all()?;

                // Explicitly drop `buffer_file` before rename so the OS file handle is closed first
                drop(buffer_file);

                // Rename `.swap` -> `.jsonl`
                self.derive_json_file()?;
            }
        }

        // Cleanly reset all per-ticket state back to `None`; `logs_dir` remains untouched!
        self.log_state_file = None;
        self.state_ticket_id = None;

        Ok(())
    }

    pub fn fetch_log_entry(&self, ticket_id: &str) -> io::Result<Vec<LogEntry>> {
        // fetch the logs from the file and return iterator expression of
        // every log entry it could convert from that jsonl file !!

        let mut file_address = PathBuf::from(&self.logs_dir);
        file_address.push(ticket_id);
        file_address.set_extension("jsonl");

        // if the jsonl file doesnt exists return Error else return Log Entry stream  !

        if file_address.exists() && file_address.is_file() {
            // return serialized Log Entry Objects

            let mut log_entry_list_completed: Vec<LogEntry> = Vec::new();
            let file_handle = File::open(file_address)?;

            let reader_buffer_file = BufReader::new(file_handle);

            // iterator on readerBufferFile.lines it return every line wrapperd in result so use
            // map_while filter all the lines those are Ok
            for line in reader_buffer_file.lines().map_while(Result::ok) {
                // because lines were not serialized LogEntry they were raw String
                // we have to serialize the each line which stored log entry into single
                // log entry object using serde json !!
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                    log_entry_list_completed.push(entry);
                }
            }

            Ok(log_entry_list_completed)
        } else {
            Err(SwapLogFileWriterError::JsonFileExpectedNotFound)?
        }
    }
}
