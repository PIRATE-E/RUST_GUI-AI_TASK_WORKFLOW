use pyo3::prelude::*;

use crate::LogEntry;

/// Native C-style callback function injected into Python's `RustGuiEventHandler`.
/// WHAT: Receives the deserialized `LogEntry` directly and pushes it into the `OnceLock` channel.
/// WHY: C-functions attached to classes via `setattr` do not bind `self` in CPython.
#[pyfunction]
#[pyo3(signature=(log_entry, *_args))]
pub fn rust_handle_overrite<'py>(log_entry: LogEntry, _args: &Bound<'py, PyAny>) -> PyResult<()> {
    crate::get_global_injector_self().emit(log_entry);
    Ok(())
}

/// Monkey-patches `RustGuiEventHandler.handle` with our native Rust function.
/// WHAT CHANGED & WHY:
/// Accepts `py: Python<'_>` directly instead of calling `Python::attach`.
/// Connects the C-function directly into the class descriptor slot in CPython.
pub fn attach_handle_into_handler(py: Python<'_>) -> PyResult<()> {
    let bridge_file_s = "coldwind.slint_gui.runtime.slint_event_handler";
    let load_bridge_m = py.import(bridge_file_s)?;
    let bridge_handler_cls = load_bridge_m.getattr("RustGuiEventHandler")?;

    let handle_overrite = wrap_pyfunction!(rust_handle_overrite, py)?;
    bridge_handler_cls.setattr("handle", handle_overrite)?;

    Ok(())
}

/// Registers an instance of `RustGuiEventHandler` into coldwind-core's singleton `OnTimeRegistry`.
/// WHAT CHANGED & WHY:
/// Decoupled from message emission! Only registers the handler so tests and callers
/// can trigger any arbitrary log entries independently.
pub fn register_log_event_handler(py: Python<'_>) -> PyResult<()> {
    // 1. Ensure the handle method is monkey-patched
    attach_handle_into_handler(py)?;

    // 2. Register instance into OnTimeRegistry
    let registry_mod = py.import("coldwind.core.system_logging")?;
    let handler_mod = py.import("coldwind.slint_gui.runtime.slint_event_handler")?;

    let on_time_reg = registry_mod.getattr("OnTimeRegistry")?.call0()?;
    let my_handler = handler_mod.getattr("RustGuiEventHandler")?.call0()?;
    on_time_reg.call_method1("register", (my_handler,))?;

    Ok(())
}

// swap function for SlintGuiDisplay.display_message we have to swap that to before involk main boot
// of core coldwind.engine !!
#[pyfunction]
#[pyo3(signature=(_self, sender, content))]
pub fn swap_display_message_overrite<'py>(
    _self: &Bound<'py, PyAny>,
    sender: String,
    content: String,
) {
    println!("sender:-{sender:?} - content:-{content:?}");
}

// attach helper of display_message method
pub fn attach_display_into_handler(py: Python<'_>) -> PyResult<()> {
    let ctx_mod = py.import("coldwind.slint_gui.runtime.SlintGuiContext")?;
    let display_cls = ctx_mod.getattr("SlintGuiDisplay")?;

    let display_overrite = wrap_pyfunction!(swap_display_message_overrite, py)?;
    display_cls.setattr("display_message", display_overrite)?;

    Ok(())
}

//NOTE: demonstration for the debug_* emits
/*
Helper function to trigger coldwind-core's public `debug_info` API.
pub fn fire_debug_info(py: Python<'_>, heading: &str, body: &str) -> PyResult<()> {
    let api_mod = py.import("coldwind.core.system_logging.debug_protocol.debug_callers_api")?;
    api_mod.call_method1("debug_info", (heading, body))?;
    Ok(())
}
*/
