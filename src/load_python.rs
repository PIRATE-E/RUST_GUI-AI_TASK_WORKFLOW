use pyo3::prelude::*;

const VENV_LOCATION: &str = env!("COLDWIND_VENV");

/// Connects the active Python interpreter session to the virtual environment's site-packages.
/// WHAT CHANGED & WHY:
/// Accepts `py: Python<'_>` directly instead of calling `Python::attach` internally.
/// WHY: Adheres to the Token-Passing pattern, allowing callers to stack operations
/// within a single continuous `'py` lifetime with zero session setup overhead.
pub fn gc_connect(py: Python<'_>) -> PyResult<()> {
    let site = py.import("site")?;
    let site_pak_location = format!("{VENV_LOCATION}/lib/python3.14/site-packages");
    site.call_method1("addsitedir", (site_pak_location,))?;
    Ok(())
}

/// Loads the ColdWindEngine class object from core.
/// Returns a live `Bound<'py, PyAny>` directly without forcing a `.unbind()` into `Py<T>`.
pub fn load_core<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let core_mod_pth = "coldwind.core.engine.coldwind_engine";
    let engine_mod = py.import(core_mod_pth)?;
    let cold_wind_engine_c = engine_mod.getattr("ColdWindEngine")?;
    Ok(cold_wind_engine_c)
}

/// Instantiates the ColdWindEngine class and executes its `boot()` method.
/// Accepts the bound class reference directly from `load_core`.
pub fn turn_on_boot<'py>(
    _py: Python<'py>,
    engine_class: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let engine_inst = engine_class.call0()?;
    engine_inst.call_method0("boot")?;
    Ok(engine_inst)
}
