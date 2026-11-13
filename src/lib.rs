use core::fmt;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender, TryRecvError};
use time_format::{now, strftime_local};

// consolidate state machine which is state full fields of one ticket loop only !!
#[derive(Debug, Clone)]
pub struct TicketUiState {
    pub ticket_id: String,
    pub task_id: i32,             // represent current task on that second !!
    pub total_tasks: i32,         // Total tasks planned for this workflow
    pub rpm_count: i32,           // Sliding 1-min RPM limit tracker
    pub total_requests: i32,      // Monotonic turn lifetime counter of requests e.g 300 requests
    pub failed_count: i32,        // Recorded failure counter
    pub current_headline: String, // Active user-facing status description
}

impl Default for TicketUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl TicketUiState {
    pub fn new() -> Self {
        Self {
            // ticket id is sentinel here we didnt reqyest ticket id from rust native code instead
            // we parse it from the logs and maintain internally for validations rust already wrote
            // the ticket_id to the required component !!
            ticket_id: String::new(),
            task_id: 1,
            total_tasks: 0,
            rpm_count: 0,
            total_requests: 0,
            failed_count: 0,
            current_headline: "Initializing workflow...".to_string(),
        }
    }

    // WHAT: Resets all turn-scoped telemetry accumulators when a new ticket turn begins.
    // WHY: In DesktopStatusHandler, each turn starts with fresh counters (0 requests, 0 failures, 1 task).
    //      Without resetting, monotonic total_requests would leak from Turn 1 into Turn 2.
    pub fn reset(&mut self, new_ticket_id: String) {
        self.ticket_id = new_ticket_id;
        self.task_id = 1;
        self.total_tasks = 0;
        self.rpm_count = 0;
        self.total_requests = 0;
        self.failed_count = 0;
        self.current_headline = "Initializing workflow...".to_string();
    }

    /// Helper to format the task progress pill (e.g. "Task [2/5]")
    #[inline]
    pub fn task_progress(&self) -> String {
        if self.total_tasks >= 1 {
            format!("Task [{}/{}]", self.task_id, self.total_tasks)
        } else {
            "Task [0/0]".to_string()
        }
    }

    /// Curates telemetry from incoming Core LogEntry deltas.
    /// Parities DesktopStatusHandler.handle() 1:1.
    pub fn parse_log(&mut self, log: &LogEntry) {
        let mut had_explicit_failures = false;

        // =====================================================================
        // 1. INGEST TELEMETRY COUNTERS FROM METADATA
        // =====================================================================
        if let Some(MetaData::Dict(map)) = log.metadata.as_ref() {
            // WHAT: Latch canonical ticket_id from metadata (e.g. from TICKET • START).
            // WHY: Automatically synchronizes Thread 3 with the host platform's canonical ticket_id
            //      and resets accumulators for the new turn without manual inter-thread signaling.
            // ANDROID/PYTHON BRIDGE: Equivalent to updating active turn tag when a new Intent arrives.
            if let Some(tid) = map.get("ticket_id").and_then(|m| m.as_str()) {
                if !tid.is_empty() && tid != self.ticket_id {
                    self.reset(tid.to_string());
                }
            }

            // Task ID
            if let Some(id) = map.get("task_id").and_then(|m| m.as_i32()) {
                self.task_id = id;
            }

            // Total Tasks (standardized: total_tasks, fallback: task_count, delta: subtasks_created)
            if let Some(total) = map
                .get("total_tasks")
                .or_else(|| map.get("task_count"))
                .and_then(|m| m.as_i32())
            {
                self.total_tasks = total;
            } else if let Some(delta) = map.get("subtasks_created").and_then(|m| m.as_i32()) {
                self.total_tasks += delta;
            }

            // Dual API Counters:
            // (a) RPM sliding window (rpm_count -> requests_count -> api_calls)
            if let Some(rpm) = map
                .get("rpm_count")
                .or_else(|| map.get("requests_count"))
                .or_else(|| map.get("api_calls"))
                .and_then(|m| m.as_i32())
            {
                self.rpm_count = rpm;
            }

            // (b) Monotonic turn total requests
            // B. STRICTLY MONOTONIC Total Requests (Never decreases or drops behind!)
            if let Some(total_req) = map.get("total_requests").and_then(|m| m.as_i32()) {
                if total_req > self.total_requests {
                    self.total_requests = total_req;
                }
            }

            // (c) Monotonic failure counter
            // WHAT: Monotonic latch for turn-scoped failures.
            // WHY: Upstream subagent finalizer emits `failed_count: 0` for skipped tasks.
            //      Never allow a subsequent 0 to erase recorded API circuit-breaker failures!
            // BRIDGE: In CPython, think of this like `self.failed_count = max(self.failed_count, fails)`.
            if let Some(fails) = map
                .get("failure_count")
                .or_else(|| map.get("failed_count"))
                .and_then(|m| m.as_i32())
            {
                if fails > self.failed_count {
                    self.failed_count = fails;
                }
                had_explicit_failures = true;
            }
        }

        // Global fallback: Count ERROR / CRITICAL logs if not explicitly tracked in metadata
        if !had_explicit_failures {
            if (log.log_level == LogLevel::ERROR || log.log_level == LogLevel::CRITICAL)
                && log.message.contains("API")
            {
                self.failed_count += 1;
            }
        }

        // =====================================================================
        // 2. STATUS PRIORITY HIERARCHY (USER-FACING HEADLINE CURATION) Parse using own platform
        //    headline generator !!
        // =====================================================================
        let (heading, _body) = if let Some((h, b)) = log.message.split_once(" | ") {
            (h.trim().to_uppercase(), b.trim())
        } else {
            (log.message.trim().to_uppercase(), log.message.trim())
        };

        // Priority 1: Drop internal raw API network noise from headline
        if heading.contains("REQUEST_PROGRESS") || heading.contains("API_CALL_COMPLETED") {
            return;
        }

        // Priority 2: Extract category & tool_name from metadata if present
        let (category, tool_name) = if let Some(MetaData::Dict(map)) = log.metadata.as_ref() {
            (
                map.get("category").and_then(|m| m.as_str()),
                map.get("tool_name").and_then(|m| m.as_str()),
            )
        } else {
            (None, None)
        };

        // Priority 3: Resolve active AgentPhase from log metadata and heading
        // WHAT: Classifies incoming Core milestone into clean AgentPhase enum
        // WHY: Works whether Core uses modern category tags or legacy heading strings
        // ANDROID/PYTHON BRIDGE: Equivalent to a Kotlin `when (intent.action)` block
        let phase = if heading.contains("TOOL • EXECUTION")
            || log.log_type == LogCategory::ToolExecution
        {
            Some(status_quote_engine::AgentPhase::ToolExecution)
        } else if heading.contains("INITIAL_PLANNER")
            || category == Some("initial_planning")
            || category == Some("complexity_analysis")
            || category == Some("task_planning")
        {
            Some(status_quote_engine::AgentPhase::InitialPlanning)
        } else if heading.contains("FINALIZER") || category == Some("finalizing") {
            Some(status_quote_engine::AgentPhase::Finalizing)
        } else if heading.contains("TASK_EXECUTOR")
            || heading.contains("AGENT • STATUS")
            || heading == "STATUS"
            || category == Some("task_execution")
            || category == Some("parameter_generation")
        {
            Some(status_quote_engine::AgentPhase::TaskExecution)
        } else if heading.contains("ERROR") || category == Some("error_recovery") {
            Some(status_quote_engine::AgentPhase::ErrorRecovery)
        } else {
            None
        };

        // Priority 4: Generate indigenous quote using local StatusQuotesEngine
        if let Some(agent_phase) = phase {
            // Seed with total_requests so quotes rotate dynamically across steps
            let seed = (self.total_requests.max(self.task_id)) as u32;
            self.current_headline = status_quote_engine::StatusQuotesEngine::generate(
                agent_phase,
                Some(self.task_id),
                tool_name,
                seed,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    DEBUG,
    ERROR,
    WARNING,
    INFO,
    CRITICAL,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARNING => "WARNING",
            LogLevel::ERROR => "ERROR",
            LogLevel::CRITICAL => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogCategory {
    ApiCall,
    ToolExecution,
    AgentWorkflow,
    McpServer,
    ErrorTraceback,
    OTHER,
}

impl LogCategory {
    // if we required the str representation of every variances !

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::ApiCall => "API_CALL",
            LogCategory::ToolExecution => "TOOL_EXECUTION",
            LogCategory::AgentWorkflow => "AGENT_WORKFLOW",
            LogCategory::McpServer => "MCP_SERVER",
            LogCategory::ErrorTraceback => "ERROR_TRACEBACK",
            LogCategory::OTHER => "OTHER",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// because serde_json serialize the Enum as variance : value so then
// MetaData::Integer(10) would becomes {"Integer" : 10} in jsonl file so that is gonna create issue
// if we have to Deserialize back to LogEntry because there would be whole different keys and values
// and for metadict instead of {"task_id" : 12} it would be {"dict" : {"task_id" : {"Integer": 12}}}
// so to avoid that nesting and Variance verbose keys and value we have to use untagged
// (default is tagged)
//
//
// ❌ WITHOUT #[serde(untagged)] (Default)       ✅ WITH #[serde(untagged)]
// ───────────────────────────────────────       ──────────────────────────
// {                                             {
//   "Dict": {                                     "task_id": 2,
//     "task_id": { "Integer": 2 },                "tool": "sql"
//     "tool": { "String": "sql" }               }
//   }
// }
//
#[serde(untagged)]
pub enum MetaData {
    // If there is no metadata direct map it to Option::None
    // Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<MetaData>),
    Dict(HashMap<String, MetaData>),
}

impl MetaData {
    /// instead of having From trait i would be use helper methods !!
    /// because from traits could not be failed and i dont bother to right and match against each
    /// TryFrom permutation combination against MetaData's variances
    ///
    /// Ergonomic extractor for integer values (handles i64, float, and string int)
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            MetaData::Integer(v) => Some(*v as i32),
            MetaData::Float(v) => Some(*v as i32),
            MetaData::String(s) => s.parse::<i32>().ok(),
            _ => None,
        }
    }

    /// Ergonomic extractor for string slices
    pub fn as_str(&self) -> Option<&str> {
        match self {
            MetaData::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

// we need Recursive string representation for meta data !!

impl fmt::Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaData::String(s) => f.write_str(s),
            MetaData::Integer(i) => write!(f, "{i}"),
            MetaData::Float(fl) => write!(f, "{fl:.1}"),
            MetaData::Bool(b) => write!(f, "{b}"),
            MetaData::List(list) => {
                f.write_str("[")?;
                for (idx, item) in list.iter().enumerate() {
                    if idx > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?; // Recursive zero-alloc write!
                }
                if list.len() > 3 {
                    f.write_str(", …")?;
                }
                f.write_str("]")
            }
            MetaData::Dict(map) => {
                f.write_str("{")?;
                for (idx, (k, v)) in map.iter().enumerate() {
                    if idx > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{k}: {v}")?; // Recursive zero-alloc write!
                }
                if map.len() > 3 {
                    f.write_str(", …")?;
                }
                f.write_str("}")
            }
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for MetaData {
    type Error = PyErr;

    // PyO3 0.23+ me signature Borrowed leta hai, purane versions me &Bound<'py, PyAny>
    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // rather to have own null we would directly wrap the MetaData as the Option
        // 1. O(1) Pointer check: ob.as_ptr() == Py_None()
        // if ob.is_none() {
        //     return Ok(MetaData::Null);
        // }

        // 2. Logs me Strings sabse zyada common hote hain — check this first!
        // cast_exact sirf pointer compare karta hai (ob->ob_type == &PyUnicode_Type)
        if let Ok(s) = ob.cast_exact::<PyString>() {
            return Ok(MetaData::String(s.to_cow()?.into_owned()));
        }

        // 3. Exact Integer Check (ob->ob_type == &PyLong_Type)
        if let Ok(i) = ob.cast_exact::<PyInt>() {
            return Ok(MetaData::Integer(i.extract::<i64>()?));
        }

        // 4. Exact Bool Check (ob->ob_type == &PyBool_Type)
        if let Ok(b) = ob.cast_exact::<PyBool>() {
            return Ok(MetaData::Bool(b.is_true()));
        }

        // 5. Exact Float Check (ob->ob_type == &PyFloat_Type)
        if let Ok(f) = ob.cast_exact::<PyFloat>() {
            return Ok(MetaData::Float(f.extract::<f64>()?));
        }

        // 6. Dict Check with pre-allocated capacity
        if let Ok(dict) = ob.cast::<PyDict>() {
            let mut map = HashMap::with_capacity(dict.len());
            for (k, v) in dict.iter() {
                let key = if let Ok(k_str) = k.cast_exact::<PyString>() {
                    k_str.to_cow()?.into_owned()
                } else {
                    k.str()?.to_cow()?.into_owned()
                };
                // Recursive step: extract<MetaData>() call karta hai bina failure overhead ke
                map.insert(key, v.extract::<MetaData>()?);
            }
            return Ok(MetaData::Dict(map));
        }

        // 7. List Check with pre-allocated capacity
        if let Ok(list) = ob.cast::<PyList>() {
            let mut vec = Vec::with_capacity(list.len());
            for item in list.iter() {
                vec.push(item.extract::<MetaData>()?);
            }
            return Ok(MetaData::List(vec));
        }

        // 8. Fallback: custom objects (datetime, UUID, etc.) string representation me convert honge
        Ok(MetaData::String(ob.str()?.to_cow()?.into_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // to make LogEntry serialized into bytes to save into file !!
pub struct LogEntry {
    pub log_type: LogCategory,
    pub log_level: LogLevel,
    pub timestamp: String,
    pub message: String,
    pub metadata: Option<MetaData>,
}

//
//

impl<'a, 'py> FromPyObject<'a, 'py> for LogEntry {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        //
        // -- policy for now that the serialization of hard types could be fail fast !!
        // -- soft could be stand out by deriving default on err or option !!
        //
        // hard Enums and their value !!
        let raw_str_log_type: String = obj.getattr("LOG_TYPE")?.getattr("value")?.extract()?;
        let raw_str_log_level: String = obj.getattr("LOG_LEVEL")?.getattr("value")?.extract()?;

        // hard actual message !!
        let message: String = obj.getattr("MESSAGE")?.extract()?;

        // soft !! time stamps !! default factory by us !!
        // time formate is "%d-%m-%Y %I:%M:%S.%f" !!
        let time_stamps = {
            match obj.getattr("TIME_STAMP") {
                Ok(time_st_rs) => match time_st_rs.extract::<String>() {
                    Ok(str_time) => str_time,
                    Err(_) => strftime_local("%d-%m-%Y %I:%M:%S.%f", now().unwrap()).unwrap(),
                },
                Err(_) => strftime_local("%d-%m-%Y %I:%M:%S.%f", now().unwrap()).unwrap(),
            }
        };
        // hard de serialization of metadata
        let meta_data = {
            match obj.getattr("METADATA") {
                Ok(m) => {
                    if m.is_none() {
                        None
                    } else {
                        m.extract::<MetaData>().ok()
                    }
                }
                Err(_) => None,
            }
        };
        // match for the log level and category !

        Ok(LogEntry {
            log_type: match raw_str_log_type.as_str() {
                "API_CALL" => LogCategory::ApiCall,
                "TOOL_EXECUTION" => LogCategory::ToolExecution,
                "AGENT_WORKFLOW" => LogCategory::AgentWorkflow,
                "MCP_SERVER" => LogCategory::McpServer,
                "ERROR_TRACEBACK" => LogCategory::ErrorTraceback,
                _ => LogCategory::OTHER,
            },
            log_level: match raw_str_log_level.as_str() {
                "DEBUG" => LogLevel::DEBUG,
                "WARNING" => LogLevel::WARNING,
                "ERROR" => LogLevel::ERROR,
                "CRITICAL" => LogLevel::CRITICAL,
                _ => LogLevel::INFO,
            },
            timestamp: time_stamps,
            message,
            metadata: meta_data,
        })
    }
}

pub struct InjectorLogEntry(SyncSender<LogEntry>);
pub struct ReceiverLogEntry(pub Receiver<LogEntry>);

impl ReceiverLogEntry {
    #[inline]
    pub fn recv(&self) -> Result<LogEntry, RecvError> {
        self.0.recv()
    }

    #[inline]
    pub fn try_recv(&self) -> Result<LogEntry, TryRecvError> {
        self.0.try_recv()
    }

    #[inline]
    pub fn into_inner(self) -> Receiver<LogEntry> {
        self.0
    }
}

impl InjectorLogEntry {
    // making this inline because of hot function to avoid pointer jumps over the binary function
    // address instead embed the emit directly in to the callers
    #[inline]
    fn emit(&self, log: LogEntry) {
        self.0.send(log).expect("oops emit failed !!");
    }
}

// NOTE:
// because the sync sender have to get init on run time but rust is static in nature
// so we cant make it like that on runtime init and define the body and define global
// to emit (we need self) and self we need to init !! but init is restricted on runtime for
// allocate the capacity of the queue we have to make it (Write Once Initialization Cell)
// which would gonna create the self of the InjectorLogEntry at runtime !!
pub static GLOBAL_INJECTOR_LOG_ENTRY: OnceLock<InjectorLogEntry> = OnceLock::new();

pub fn get_global_injector_self() -> &'static InjectorLogEntry {
    GLOBAL_INJECTOR_LOG_ENTRY
        .get()
        .expect("InjectorLogEntry is not initialized!!")
}

// initialize the sync_channel and set the global object of Sender and return the receiver out of
// the initialization success !!
pub fn init_sync_send_all() -> ReceiverLogEntry {
    let (sender, receiver) = mpsc::sync_channel::<LogEntry>(1024);

    let _ = GLOBAL_INJECTOR_LOG_ENTRY.set(InjectorLogEntry(sender));
    ReceiverLogEntry(receiver)
}

// grab the sync sender's receiver when we perform the initialization of the InjectorLogEntry
// instead of making it also global

// this is key file got all the handle helper to
// connect coldwind {load coldwind directly in the running env}
pub mod load_python;
pub mod log_entry_handler;
pub mod serialize_logs_in_file;
pub mod slint_py_engine;
pub mod status_quote_engine;
