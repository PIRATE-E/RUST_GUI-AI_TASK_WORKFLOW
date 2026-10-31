//! =============================================================================
//! FILE: tests/test_log_entry_extract.rs
//! PURPOSE: Level 2 integration test validating Python -> Rust log sink deserialization.
//!
//! TESTED MODULES:
//! - [src/lib.rs](file:///home/pirate/development/ai-workflow-gui/src/lib.rs):
//!   - `init_sync_send_all()`: Creates global `OnceLock` MPSC channel for logging events
//!   - `LogEntry`, `LogLevel`, `LogCategory`, `MetaData`: Typed Rust deserialization models
//! - [src/log_entry_handler.rs](file:///home/pirate/development/ai-workflow-gui/src/log_entry_handler.rs):
//!   - `attach_handle_into_handler(py)`: Injects native Rust descriptor into `RustGuiEventHandler.handle`
//!   - `register_log_event_handler(py)`: Registers the patched handler into Core's `OnTimeRegistry`
//! - Upstream Python:
//!   - `coldwind.core.system_logging.debug_protocol.debug_callers_api.debug_info`
//!
//! ARCHITECTURAL INVARIANT TESTED:
//! Tests "In-Memory Platform Log Sinking":
//! Proves that Core logging events emitted via Python's public `debug_info(...)` API
//! bypass raw OS files/sockets and travel through:
//! `debug_info` -> `OnTimeRegistry` -> `RustGuiEventHandler.handle` (Python descriptor)
//! -> `rust_handle_overrite` (Rust pyfunction) -> MPSC channel -> Rust receiver.
//! =============================================================================

use ai_workflow_gui::{init_sync_send_all, load_python::gc_connect, log_entry_handler};
use pyo3::prelude::*;

/// Verifies that:
/// 1. `init_sync_send_all()` establishes a synchronized MPSC channel.
/// 2. `register_log_event_handler(py)` patches `RustGuiEventHandler` and binds it to `OnTimeRegistry`.
/// 3. Calling `debug_info(...)` in Python serializes the log entry and triggers the native Rust callback.
/// 4. The native Rust receiver extracts the strongly typed `LogEntry` with matching message content.
#[test]
fn test_direct_debug_convert() -> PyResult<()> {
    // 1. Initialize global logging channel and grab the receiver handle
    let receiver = init_sync_send_all();

    // 2. Open a single Python session and stack components using token passing
    Python::attach(|py| -> PyResult<()> {
        gc_connect(py)?;
        log_entry_handler::register_log_event_handler(py)?;

        // 3. Dispatch a real Python debug log entry via Core's public API
        let debug_api = py.import("coldwind.core.system_logging.debug_protocol.debug_callers_api")?;
        debug_api.call_method1("debug_info", ("LiveTest", "Model booted successfully"))?;

        Ok(())
    })?;

    // 4. Drain the MPSC channel on the Rust side and assert payload fidelity
    let entry = receiver
        .recv()
        .expect("Failed to receive log entry from Python through native MPSC channel");

    println!(
        "[test_log_entry_extract] Received native LogEntry: [{:?}] {} (has metadata: {})",
        entry.log_level,
        entry.message,
        entry.metadata.is_some()
    );

    assert_eq!(
        entry.message,
        "LiveTest | Model booted successfully",
        "Deserialized message string must match Python debug_info input exactly"
    );

    Ok(())
}
