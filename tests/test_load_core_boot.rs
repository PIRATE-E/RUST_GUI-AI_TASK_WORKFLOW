//! =============================================================================
//! FILE: tests/test_load_core_boot.rs
//! PURPOSE: Level 1 FFI test validating ColdWindEngine class loading and boot-order invariant.
//!
//! TESTED MODULES:
//! - [src/load_python.rs](file:///home/pirate/development/ai-workflow-gui/src/load_python.rs):
//!   - `gc_connect(py)`: Connects Python interpreter to upstream `.venv/site-packages`
//!   - `load_core(py)`: Dynamically imports and returns `ColdWindEngine` class handle
//!   - `turn_on_boot(py, &class)`: Instantiates class and calls `boot()`
//! - Upstream Python class: `coldwind.core.engine.coldwind_engine.ColdWindEngine`
//!
//! ARCHITECTURAL INVARIANT TESTED:
//! Tests the "ContextRegistry Boot-Order Invariant":
//! `ColdWindEngine.boot()` requires a registered `PlatformRuntimeContext` (such as
//! `DesktopRunTimeContext` or `SlintRuntimeContext`). If instantiated and booted bare
//! without an active context, it MUST fail fast and raise an exception. This test
//! verifies that the class loads, instantiates, and fails cleanly without crashing
//! the host process.
//! =============================================================================

use ai_workflow_gui::load_python::{gc_connect, load_core, turn_on_boot};
use pyo3::prelude::*;

/// Verifies that:
/// 1. `load_core` extracts the live `ColdWindEngine` class object from Python.
/// 2. `turn_on_boot` attempts instantiation and executes `.boot()`.
/// 3. Without platform context activation, the boot call fails cleanly and the
///    Python exception traceback can be inspected and extracted by Rust.
#[test]
fn test_load_core_boot() -> PyResult<()> {
    Python::attach(|py| {
        // 1. Connect Python virtualenv site-packages
        gc_connect(py)?;

        // 2. Load the ColdWindEngine class object
        let engine_class = load_core(py)?;
        assert!(!engine_class.is_none(), "ColdWindEngine class reference must be valid");

        // 3. Attempt bare boot without activating a platform context
        // NOTE: We expect an Err because ColdWindEngine requires ContextRegistry.activate_context()
        let load_res = turn_on_boot(py, &engine_class);

        // 4. Assert fail-fast behavior and inspect Python exception traceback
        assert!(
            load_res.is_err(),
            "ColdWindEngine::boot() must fail fast when executed without an active PlatformRuntimeContext!"
        );

        if let Err(load_err) = load_res {
            let tb_mod = py.import("traceback")?;
            let line_s: Vec<String> = tb_mod
                .call_method1("format_exception", (load_err,))
                .and_then(|res| res.extract::<Vec<String>>())
                .unwrap_or_else(|err| vec![err.to_string()]);

            let error_traceback: String = line_s.into_iter().collect();
            println!("[test_load_core_boot] Verified expected fail-fast exception:\n{error_traceback}");
            assert!(!error_traceback.is_empty(), "Traceback string must not be empty");
        }

        Ok(())
    })
}
