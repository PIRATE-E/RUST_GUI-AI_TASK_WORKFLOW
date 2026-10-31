//! =============================================================================
//! FILE: tests/test_build_checkup.rs
//! PURPOSE: Level 0 smoke test validating PyO3 CPython 3.14 FFI linking and venv discovery.
//!
//! TESTED MODULES:
//! - `.cargo/config.toml` (Environment keys: `PYO3_PYTHON`, `COLDWIND_VENV`)
//! - Upstream Python environment: `/home/pirate/development/ai-workflow-task-agent/.venv`
//! - `coldwind.core` namespace package root
//!
//! ARCHITECTURAL ROLE:
//! Serves as the ground-floor foundation test for the entire desktop application.
//! Verifies that:
//! 1. Rust can successfully link against the embedded `libpython3.14.so` runtime.
//! 2. Python's `site` module can dynamically mount the external virtual environment's
//!    `site-packages` directory via `addsitedir()`.
//! 3. The `coldwind.core` package can be imported directly across workspace boundaries
//!    without needing to execute a full application boot.
//! =============================================================================

use pyo3::prelude::*;

/// Validates that CPython 3.14 initializes cleanly, injects the `COLDWIND_VENV`
/// site-packages directory into `sys.path`, and successfully imports `coldwind.core`.
#[test]
fn test_build_check_up() -> PyResult<()> {
    // 1. Read virtual environment root configured by .cargo/config.toml
    let venv_location = env!("COLDWIND_VENV");

    Python::attach(|py| {
        // 2. Resolve the Python 3.14 site-packages directory
        let site_package = format!("{venv_location}/lib/python3.14/site-packages");
        let site = py.import("site")?;
        
        // 3. Dynamically append site-packages (enables importing upstream packages)
        site.call_method1("addsitedir", (site_package,))?;

        // 4. Verify that coldwind.core can be imported cleanly
        let core = py.import("coldwind.core")?;
        println!("[test_build_checkup] Successfully imported: {core:?}");

        // 5. Verify package attributes exist
        assert!(core.getattr("__file__").is_ok() || core.getattr("__path__").is_ok());

        Ok(())
    })
}
