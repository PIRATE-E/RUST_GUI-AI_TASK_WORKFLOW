# 🦀 Rust Core Business Logic, Concurrency & FFI Architecture (`src/`)

| Metric | Specification |
| :--- | :--- |
| **Domain** | Native Rust Business Logic, Actor Concurrency & PyO3 0.26 FFI |
| **Path** | `reports/system-architecture/src/README.md` |
| **Target Audience** | Python & Systems Engineers (CPython 3.14 internals, GIL, `queue.Queue`, Actor Pattern, PyO3) |
| **Files Documented** | `lib.rs`, `main.rs`, `load_python.rs`, `log_entry_handler.rs`, `slint_py_engine.rs`, `status_quote_engine.rs`, `serialize_logs_in_file.rs`, `splash_screen.rs` |
| **Status** | 🟢 Complete (8 Subsystems Documented) |
| **Last Updated** | 2026-11-15 |

> **TL;DR:** This domain provides an exhaustive, section-by-section breakdown of every Rust source file in `src/`. It explains how `ai-workflow-gui` achieves 144 Hz responsive UI rendering while executing long-running Python LangGraph AI workflows through a 3-thread actor pipeline (Threads 0, 2, 3), zero-copy FFI extraction, and indigenous quote telemetry.

---

## 🗺️ 1. Systems Architecture & Thread Topology

The Rust core logic coordinates three independent OS threads with zero shared mutable state:

```text
+-----------------------------------------------------------------------------+
|                      RUST CORE SUBSYSTEM TOPOLOGY                           |
+-----------------------------------------------------------------------------+

             Thread 0 (UI + 144Hz)              Thread 2 (Worker Actor)
       +-----------------------+              +-----------------------+
       | src/main.rs           |              | src/slint_py_engine.rs|
       | • AppWindow event loop|  sync_channel| • SlintRuntime facade |
       | • VecModel in-place   | ────────────►| • process_ticket()    |
       | • Uuid::new_v4() stamp|      (1)     | • CPython GIL holder  |
       +-----------▲-----------+              +-----------┬-----------+
                   │                                      │
                   │ slint::invoke_from_event_loop        │ Imports & Boots
                   │                                      ▼
                   │                          +-----------------------+
                   │                          | src/load_python.rs    |
                   │                          | • gc_connect (.venv)  |
                   │                          | • turn_on_boot        |
                   │                          +-----------┬-----------+
                   │                                      │
                   │                                      │ Intercepts Logs
                   │                                      ▼
                   │                          +-----------------------+
                   │                          | src/log_entry_handler |
                   │                          | • Trampoline patch    |
                   │                          | • rust_handle_overrite|
                   │                          +-----------┬-----------+
                   │                                      │
                   │                                      │ Emits LogEntry
                   │                                      ▼
             Thread 3 (Sink)                  +-----------------------+
       +-----------------------+  sync_channel| src/lib.rs            |
       | src/main.rs           |◄─────────────| • InjectorLogEntry    |
       | • TicketUiState loop  |    (1024)    | • OnceLock global     |
       | • AgentPhase mapping  |              | • MetaData unboxing   |
       +-----------┬-----------+              +-----------------------+
                   │
                   │ Generates Quotes Locally
                   ▼
       +-----------------------+
       | src/status_quote_     |
       | engine.rs             |
       | • Zero-GIL quotes     |
       +-----------------------+
```

---

## 📈 2. Recommended Reading Path

To master the systems and FFI architecture, read the documents in this sequence:

```text
  1. Core Types & Channels  2. Main Concurrency   3. CPython Boot & Venv
+-------------------------+ +-------------------+ +-----------------------+
| 01_lib_core_types_      |►| 02_main_          |►| 03_cpython_boot_and_  |
| and_channels.md         | | orchestrator.md   | | venv_injection.md     |
+-------------------------+ +-------------------+ +-----------------------+
                                                              │
                                                              ▼
  6. Status Quotes Engine   5. Engine Facade      4. Descriptor Patching
+-------------------------+ +-------------------+ +-----------------------+
| 06_status_quote_        |◄| 05_slint_py_      |◄| 04_pyo3_descriptor_   |
| engine.md               | | engine_runtime.md | | monkeypatching.md     |
+-------------------------+ +-------------------+ +-----------------------+
```

1. [**01_lib_core_types_and_channels.md**](./01_lib_core_types_and_channels.md): Learn about `MetaData` recursive extraction, `TicketUiState` telemetry state machine, and the `OnceLock`-backed bounded `sync_channel(1024)`.
2. [**02_main_orchestrator_and_multithreading.md**](./02_main_orchestrator_and_multithreading.md): Master the 3-thread concurrency topology, the `sync_channel(1)` actor loop, primary key immutability, and zero-flicker in-place bubble mutation.
3. [**03_cpython_boot_and_venv_injection.md**](./03_cpython_boot_and_venv_injection.md): Understand how `gc_connect` injects the virtual environment via `site.addsitedir()` and how the Token-Passing (`Python<'py>`) lifetime pattern eliminates GIL overhead.
4. [**04_pyo3_descriptor_monkeypatching.md**](./04_pyo3_descriptor_monkeypatching.md): Explore dynamic class monkey-patching, C-API trampolines via `wrap_pyfunction!`, and class-level event interception.
5. [**05_slint_py_engine_runtime_facade.md**](./05_slint_py_engine_runtime_facade.md): Study the `SlintRuntime` facade, cached class/tool handles, $O(1)$ heap pointer swapping, and 0ms fast-path intercepts for `/clear` and `exit`.
6. [**06_status_quote_engine.md**](./06_status_quote_engine.md): Discover how `StatusQuotesEngine` provides indigenous UI personality and humor with zero GIL contention and sub-microsecond latency.
7. [**03-disk-persistence-overview.md**](../03-disk-persistence-overview.md): Covers `src/serialize_logs_in_file.rs`, telemetry JSON serialization, and ACID `.swap` to `.jsonl` disk persistence.
8. [**07-ascii-splash-screen-engine.md**](../07-ascii-splash-screen-engine.md): Covers `src/splash_screen.rs`, embedded ZSTD Kyoto Dusk splash asset decompression, 144 Hz timer pacing, Fontdue glyph caching, and gamma LUT.

---

## 📋 3. Document Index (At a Glance)

| Document | Type | Core Systems Question Answered | Python Mental Anchor | Status |
| :--- | :--- | :--- | :--- | :--- |
| [**01_lib_core_types_and_channels.md**](./01_lib_core_types_and_channels.md) | Systems Note | How are Python dictionaries extracted without reflection? | `pydantic.BaseModel` & `queue.Queue(1024)` | 🟢 Complete |
| [**02_main_orchestrator_and_multithreading.md**](./02_main_orchestrator_and_multithreading.md) | Systems Note | How do 3 OS threads communicate with zero mutex locks? | `threading.Thread` & Actor Loop | 🟢 Complete |
| [**03_cpython_boot_and_venv_injection.md**](./03_cpython_boot_and_venv_injection.md) | Systems Note | How does embedded CPython find `.venv` site-packages? | `site.addsitedir()` & Token-Passing | 🟢 Complete |
| [**04_pyo3_descriptor_monkeypatching.md**](./04_pyo3_descriptor_monkeypatching.md) | Systems Note | How does Rust intercept Python methods at runtime? | Descriptor protocol (`__get__`) & `setattr` | 🟢 Complete |
| [**05_slint_py_engine_runtime_facade.md**](./05_slint_py_engine_runtime_facade.md) | Systems Note | How does Rust execute turns without re-importing classes? | `TicketIn(**kwargs)` & Heap Swapping | 🟢 Complete |
| [**06_status_quote_engine.md**](./06_status_quote_engine.md) | Systems Note | Why did we decouple quotes from Python `agent_status.py`? | Zero-GIL static arrays & Modulo rotation | 🟢 Complete |
| [**03-disk-persistence-overview.md**](../03-disk-persistence-overview.md) | Systems Note | How does `src/serialize_logs_in_file.rs` ensure ACID disk persistence? | `jsonl` atomic rename & `fsync` | 🟢 Complete |
| [**07-ascii-splash-screen-engine.md**](../07-ascii-splash-screen-engine.md) | Systems Note | How does `src/splash_screen.rs` rasterize 1,152 ASCII frames at 144 Hz? | ZSTD streaming & glyph caching | 🟢 Complete |

---

## 🌉 4. The CPython ↔ Rust Native Rosetta Stone

| CPython 3.14 Concept | Rust Native Equivalent in `src/` | Critical Systems Insight |
| :--- | :--- | :--- |
| `dict[str, Any]` | `pub enum MetaData` | Type-erased FFI deserialization with zero pointer leaks. |
| `queue.Queue(maxsize=1)` | `mpsc::sync_channel::<WorkerState>(1)` | Enforces strict 1-turn-at-a-time actor sequencing. |
| `site.addsitedir()` | `site.call_method1("addsitedir", ...)` | Injects editable virtual environment packages. |
| `cls.method = fn` | `bridge_cls.setattr("handle", wrap_pyfunction!(fn, py)?)` | Injects compiled native C trampoline into class descriptor slot. |
| `state = new_state` | `self.session_state = new_state.unbind()` | $O(1)$ pointer assignment on the Python heap. |
| `random.choice(quotes)` | `(turn_count as usize) % QUOTES.len()` | Deterministic, lock-free quote selection with 0 GIL cost. |
