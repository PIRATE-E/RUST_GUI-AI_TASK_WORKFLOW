use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender, TryRecvError};
use time_format::{now, strftime_local};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    ERROR,
    WARNING,
    INFO,
    CRITICAL,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogCategory {
    ApiCall,
    ToolExecution,
    AgentWorkflow,
    McpServer,
    ErrorTraceback,
    OTHER,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
pub mod slint_py_engine;
