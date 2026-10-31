//! =============================================================================
//! FILE: tests/test_part1_ticket.rs
//! PURPOSE: Level 3 end-to-end integration test validating the complete headless ticket loop.
//!
//! TESTED MODULES:
//! - [src/slint_py_engine.rs](file:///home/pirate/development/ai-workflow-gui/src/slint_py_engine.rs):
//!   - `SlintRuntime::boot(py)`: Boots upstream orchestrator, context, and LangGraph engine
//!   - `SlintRuntime::process_ticket(py, prompt, timeout)`: Constructs `TicketIn` and unpacks `TicketOut`
//! - [src/log_entry_handler.rs](file:///home/pirate/development/ai-workflow-gui/src/log_entry_handler.rs):
//!   - `attach_handle_into_handler(py)`: Injects native logging descriptor before boot
//! - Upstream Python:
//!   - `coldwind.slint_gui.main_orchestrator.boot_slint_py_runtime`
//!   - `coldwind.core.engine.coldwind_engine.ColdWindEngine`
//!   - `coldwind.core.models.ticket_in.TicketIn`
//!   - `coldwind.core.models.ticket_out.TicketOut`
//!
//! ARCHITECTURAL INVARIANT TESTED:
//! Tests "The Complete Turn Lifecycle Invariant":
//! 1. Native descriptor swapping must precede `boot_slint_py_runtime()` to capture boot logs.
//! 2. `SlintRuntime::boot(py)` successfully activates `SlintRuntimeContext`, loads `.env.slint`,
//!    and instantiates `ColdWindEngine`.
//! 3. `process_ticket(...)` executes a full LangGraph turn (router -> chatBot), respects
//!    slash commands (e.g. `/help`), and unpacks `TicketOutResult` containing status,
//!    latency, and transcript without crashing the Python GIL session.
//! =============================================================================

use ai_workflow_gui::{
    init_sync_send_all, load_python::gc_connect, log_entry_handler::attach_handle_into_handler,
    slint_py_engine::SlintRuntime,
};
use pyo3::prelude::*;

/// Executes a complete end-to-end ticket turn against ColdWind Core:
/// 1. Initializes the native logging MPSC channel.
/// 2. Attaches to Python, adds virtual environment site-packages, and patches descriptors.
/// 3. Boots `SlintRuntime` through `main_orchestrator.boot_slint_py_runtime()`.
/// 4. Submits a ticket turn with `/help` command and verifies `status == "COMPLETED"`.
#[test]
fn test_part1_ticket_turn() -> PyResult<()> {
    // 1. Initialize native logging channel to capture boot-time events
    let _receiver = init_sync_send_all();

    Python::attach(|py| {
        // 2. Connect Python virtual environment site-packages
        gc_connect(py)?;

        // 3. Monkey-patch logging handler descriptor before booting the core
        attach_handle_into_handler(py)?;

        // 4. Boot SlintRuntime via main_orchestrator
        let mut runtime = SlintRuntime::boot(py)?;

        // 5. Submit an end-to-end ticket turn (/help command)
        let result = runtime.process_ticket(py, "/help", 30)?;

        println!("[test_part1_ticket_turn] Turn Status: {}", result.status);
        println!("[test_part1_ticket_turn] Transcript: {}", result.transcript);
        println!("[test_part1_ticket_turn] Turn Latency: {} ms", result.latency_ms);

        // 6. Assertions on the TicketOut contract
        assert_eq!(result.status, "COMPLETED", "Ticket status must be COMPLETED");
        assert!(!result.transcript.is_empty(), "AI transcript must not be empty");
        assert!(!result.is_exiting, "Ticket turn should not trigger exit on /help");
        assert!(result.latency_ms >= 0.0, "Turn latency must be non-negative");

        Ok(())
    })
}
