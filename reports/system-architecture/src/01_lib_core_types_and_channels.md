# 🧱 Rust Core Types, Zero-Copy FFI Extraction & Channel Architecture (`src/lib.rs`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`src/lib.rs (core types & log channel)`](../../../src/lib.rs) |
| **Document Path** | `reports/system-architecture/src/01_lib_core_types_and_channels.md` |
| **Domain** | Rust Core Logic & Systems Architecture |
| **Target Persona** | Python & Systems Engineer (CPython 3.14 internals, GIL, `PyObject*`, `threading`, `queue.Queue`) |
| **Python Mental Anchor** | CPython C-API FFI, `queue.Queue(maxsize=1024)`, `dataclasses`, `enum.Enum`, `ctypes` |
| **Verification Mode** | Codebase Verified ([`src/lib.rs (core types & log channel)`](../../../src/lib.rs)) |
| **Confidence** | VERIFIED |

> **TL;DR:** `src/lib.rs` is the nervous system of `ai-workflow-gui`. It defines the core data pipeline: recursive type-erased Python metadata extraction (`FromPyObject<'a, 'py>`), the `TicketUiState` live telemetry accumulator with autonomous phase quote dispatching, and a high-performance `OnceLock`-backed bounded `sync_channel(1024)` bridging Python log hooks to Rust consumer threads.

---

## 🏛️ 1. Architecture & Data Flow Overview

In a Python-centric architecture, you might emit logs to a thread-safe `queue.Queue` or a logging handler attached to the standard `logging` library. In our native desktop application, Python code running inside embedded CPython 3.14 needs to transmit structured logging events across the FFI boundary into Rust without holding the Global Interpreter Lock (GIL) or blocking the UI.

```text
+-----------------------------------------------------------------------------+
|                     src/lib.rs DATA PROCESSING PIPELINE                     |
+-----------------------------------------------------------------------------+

  CPython 3.14 Log Emitter               FFI Extraction Layer (FromPyObject)
 +------------------------+              +------------------------------------+
 | Python LogEntry Object | ------------►| impl FromPyObject for MetaData    |
 | • LOG_TYPE (Enum)      |   O(1) Ptr   | • O(1) PyUnicode_Check exact match |
 | • LOG_LEVEL (Enum)     |   Unboxing   | • Recursive PyDict & PyList unbox  |
 | • MESSAGE (str)        |              +-----------------+------------------+
 | • METADATA (dict/None) |                                |
 +------------------------+                                ▼ Produces Native
                                                 +----------------------------+
                                                 | Rust LogEntry Struct       |
                                                 | (Send + Sync + 'static)    |
                                                 +-------------+--------------+
                                                               |
                     Emitted via Global Channel                ▼
 +-------------------------------------------------------------+--------------+
 | GLOBAL_INJECTOR_LOG_ENTRY: OnceLock<InjectorLogEntry>                      |
 | Uses bounded std::sync::mpsc::sync_channel::<LogEntry>(1024)               |
 +-----------------------------+----------------------------------------------+
                               |
                               ▼ Received on Dedicated Thread 3
 +-----------------------------+----------------------------------------------+
 | TicketUiState::parse_log(&log)                                             |
 | • Ingests RPM, Total Requests, Task Progress, and Error counters           |
 | • Drops raw internal network noise (REQUEST_PROGRESS)                      |
 | • Classifies AgentPhase -> dispatches to StatusQuotesEngine                |
 +----------------------------------------------------------------------------+
```

### The CPython ↔ Rust Mental Bridge

| Python / CPython 3.14 Concept | Rust `src/lib.rs` Reality | Systems Advantage |
| :--- | :--- | :--- |
| `dict[str, Any]` (Heterogeneous dynamic metadata) | `pub enum MetaData` (Recursive tagged union) | 100% type-safe in Rust; eliminates `PyObject*` reference leaks. |
| `pydantic.BaseModel.model_validate(obj)` | `impl<'a, 'py> FromPyObject<'a, 'py> for LogEntry` | Compiles directly into native machine code pointer checks with 0 runtime reflection. |
| `queue.Queue(maxsize=1024)` | `std::sync::mpsc::sync_channel::<LogEntry>(1024)` | Bounded ring-buffer channel providing non-blocking backpressure across thread boundaries. |
| Module-level singleton: `_queue = None` | `pub static GLOBAL_INJECTOR_LOG_ENTRY: OnceLock<InjectorLogEntry>` | Thread-safe, lock-free once-initialization cell; zero mutex contention after init. |
| `DesktopStatusHandler` state machine | `pub struct TicketUiState` | Encapsulates all mutable telemetry metrics into an unsynchronized stack struct on Thread 3. |

---

## 🔍 2. Keyword & Systems Decoder

| Rust Keyword / Type | CPython / C-API Analog | Plain-English Systems Explanation |
| :--- | :--- | :--- |
| `Borrowed<'a, 'py, PyAny>` | Borrowed `PyObject*` reference | A Python object pointer that is valid for the lifetime `'py` of the GIL token. Does not increment the reference count (`Py_INCREF` is avoided). |
| `cast_exact::<PyString>()` | `ob->ob_type == &PyUnicode_Type` | Direct pointer comparison against Python's built-in type object. O(1) instantaneous check without walking subclass hierarchies. |
| `OnceLock<T>` | Thread-safe atomic pointer check | A container that can be written to exactly once. Subsequent reads are pure lock-free atomic pointer dereferences. |
| `sync_channel(1024)` | Bounded FIFO queue | Bounded multi-producer, single-consumer channel. If the queue reaches 1024 items, sending threads block or drop rather than exhausting memory. |
| `#[inline]` | Compiler hint `__attribute__((always_inline))` | Embeds function machine code directly at call sites to eliminate call-instruction overhead on hot loops. |

---

## 🧱 3. Data Structures & Extraction Logic

### 3.1 The Recursive `MetaData` Tagged Union

In Python, log metadata is an arbitrary dictionary: `{"task_id": 1, "rpm_count": 45, "tools": ["web_search", "eval"]}`. Because Rust requires strict compile-time types, `src/lib.rs` models dynamic Python dictionaries using an enum:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MetaData {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<MetaData>),
    Dict(HashMap<String, MetaData>),
}
```

#### The Fast-Path `FromPyObject` Extraction Strategy

Lines 263–322 implement custom FFI deserialization. Instead of using generic slow reflection, it tests types in order of statistical frequency in log streams:

```rust
// 1. Strings are the most common field in logs: check PyUnicode first!
if let Ok(s) = ob.cast_exact::<PyString>() {
    return Ok(MetaData::String(s.to_cow()?.into_owned()));
}
// 2. Integers (task IDs, RPM counters)
if let Ok(i) = ob.cast_exact::<PyInt>() {
    return Ok(MetaData::Integer(i.extract::<i64>()?));
}
// 3. Dictionaries with pre-allocated capacity
if let Ok(dict) = ob.cast::<PyDict>() {
    let mut map = HashMap::with_capacity(dict.len());
    for (k, v) in dict.iter() {
        // Recursively unpacks values
        map.insert(key, v.extract::<MetaData>()?);
    }
    return Ok(MetaData::Dict(map));
}
```

| Type Check Order | Target Python Type | Systems Optimization |
| :--- | :--- | :--- |
| **1. String** | `str` (`PyUnicode`) | `cast_exact` matches pointer directly; copies via copy-on-write `to_cow()`. |
| **2. Int** | `int` (`PyLong`) | Unpacks 64-bit integer directly. |
| **3. Bool** | `bool` (`PyBool`) | Evaluates `ob.is_true()`. Must precede `PyInt` because in CPython `bool` is a subclass of `int`. |
| **4. Float** | `float` (`PyFloat`) | Unpacks 64-bit IEEE float. |
| **5. Dict & List** | `dict` & `list` | Uses `with_capacity(len)` to allocate the exact memory needed in a single malloc call. |
| **6. Fallback** | Arbitrary object | Calls `ob.str()` to capture a string representation without crashing. |

---

### 3.2 `TicketUiState` Telemetry State Machine

Lines 10–206 define `TicketUiState`, which mirrors the Python core's `DesktopStatusHandler` 1:1:

```rust
#[derive(Debug, Clone)]
pub struct TicketUiState {
    pub ticket_id: String,
    pub task_id: i32,
    pub total_tasks: i32,
    pub rpm_count: i32,
    pub total_requests: i32,
    pub failed_count: i32,
    pub current_headline: String,
}
```

#### Core Operational Rules in `parse_log(&mut self, log: &LogEntry)`

1. **Noise Dropping:** Drops `REQUEST_PROGRESS` and `API_CALL_COMPLETED` log messages immediately (lines 151–153). These events occur 50+ times per second during streaming and would cause massive UI headline jitter.
2. **Monotonic `total_requests`:** Total requests only move forward (`if total_req > self.total_requests { self.total_requests = total_req; }`). This prevents out-of-order log arrival from resetting lifetime counters backward.
3. **Autonomous Phase Classification:** Maps `log.metadata["category"]` or `log.message` headings directly into `status_quote_engine::AgentPhase`:
   - `TOOL • EXECUTION` $\rightarrow$ `AgentPhase::ToolExecution`
   - `INITIAL_PLANNER` $\rightarrow$ `AgentPhase::InitialPlanning`
   - `FINALIZER` $\rightarrow$ `AgentPhase::Finalizing`
   - `TASK_EXECUTOR` $\rightarrow$ `AgentPhase::TaskExecution`
   - `ERROR` $\rightarrow$ `AgentPhase::ErrorRecovery`
4. **Local Quote Generation:** Feeds the resolved phase into [`StatusQuotesEngine::generate in status_quote_engine.rs`](../../../src/status_quote_engine.rs) to create witty, contextual quotes on Thread 3 with zero GIL contention.

---

## ⚡ 4. Channel Architecture: Bounded Zero-Lock Queue

Lines 400–450 solve the FFI log injection problem:

```rust
pub struct InjectorLogEntry(SyncSender<LogEntry>);
pub struct ReceiverLogEntry(pub Receiver<LogEntry>);

pub static GLOBAL_INJECTOR_LOG_ENTRY: OnceLock<InjectorLogEntry> = OnceLock::new();

pub fn init_sync_send_all() -> ReceiverLogEntry {
    let (sender, receiver) = mpsc::sync_channel::<LogEntry>(1024);
    let _ = GLOBAL_INJECTOR_LOG_ENTRY.set(InjectorLogEntry(sender));
    ReceiverLogEntry(receiver)
}
```

```text
 CPython Worker Thread (Thread 2)               Background Telemetry Consumer (Thread 3)
+--------------------------------+             +----------------------------------------+
| rust_handle_overrite(log)      |             | log_receiver.recv() (Blocking)         |
|   │                            |             |   │                                    |
|   ▼                            |             |   ▼                                    |
| get_global_injector_self()     |             | state.parse_log(&log)                  |
|   │                            |             |   │                                    |
|   ▼                            |             |   ▼                                    |
| OnceLock.get().unwrap().emit() |             | slint::invoke_from_event_loop(...)     |
+---------------+----------------+             +-------------------+--------------------+
                │                                                  ▲
                │ Pushes LogEntry                                  │ Updates UI State
                ▼                                                  │
+──────────────────────────────────────────────────────────────────┴────────────────────+
|        std::sync::mpsc::sync_channel::<LogEntry>(1024) Bounded Ring Buffer            |
+───────────────────────────────────────────────────────────────────────────────────────+
```

### Why Bounded `sync_channel(1024)` Over Unbounded `channel()`?

| Channel Type | Memory Behavior Under Load | UI Impact | Selected? |
| :--- | :--- | :--- | :--- |
| **`mpsc::channel()` (Unbounded)** | Python generating 10,000 logs/sec can exhaust RAM if UI thread lags. | Risk of OOM (Out Of Memory) crash. | ❌ Rejected |
| **`mpsc::sync_channel(1024)` (Bounded)** | Limits unconsumed logs to exactly 1024 items. | Constant, predictable memory footprint. Backpressure naturally paces producer. | 🟢 **Selected** |

---

## ⚠️ 5. Common Traps & CPython Pitfalls

### Trap 1: Checking `PyInt` Before `PyBool`
- **The CPython Trap:** In CPython's object model, `bool` inherits from `int` (`issubclass(bool, int) is True`).
- **The Bug:** If you test `ob.cast_exact::<PyInt>()` before `PyBool`, Python boolean values (`True`/`False`) would match as integers (`1`/`0`) in `MetaData`!
- **The Fix:** Line 280 tests `cast_exact::<PyInt>()` (exact pointer comparison), and line 286 tests `cast_exact::<PyBool>()`. Because both use `cast_exact` rather than `cast`, type collisions are strictly avoided.

### Trap 2: Holding GIL During Channel Send
- **The Systems Trap:** Emitting logs over a channel while holding the GIL.
- **The Bug:** If the receiving thread blocks or the channel fills up, holding the GIL blocks all other Python worker threads.
- **The Fix:** The log channel send operation is pure native Rust (`InjectorLogEntry.emit(log)`), executing in a few nanoseconds. The receiving thread (Thread 3) runs 100% in Rust without ever acquiring the GIL.

---

## ❓ 6. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Why is `GLOBAL_INJECTOR_LOG_ENTRY` wrapped in `OnceLock` instead of `lazy_static!`? | `OnceLock` is part of the Rust standard library since 1.70. It avoids third-party macro dependencies and provides zero-overhead lock-free reads after initialization. | 2026-11-04 |
| Q2 | What happens if `init_sync_send_all()` is called twice? | The second call will fail silently on `GLOBAL_INJECTOR_LOG_ENTRY.set(...)` because `OnceLock` rejects subsequent writes, preserving the original sender channel safely. | 2026-11-04 |
| Q3 | Why doesn't `parse_log` use regex to match log messages? | String slicing (`split_once(" \| ")`) and substring checks (`contains(...)`) are orders of magnitude faster than regex, running in sub-microsecond time on every log entry. | 2026-11-04 |
| Q4 | Which threads communicate via the log channel? | CPython Worker (Thread 2) emits logs into the bounded channel; Telemetry & ACID Log Drainer (Thread 3) drains and flushes them without touching Thread 0 UI directly. | 2026-11-15 |
