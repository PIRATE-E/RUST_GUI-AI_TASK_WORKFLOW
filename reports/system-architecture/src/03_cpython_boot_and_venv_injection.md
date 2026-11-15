# 🐍 CPython 3.14 Boot, Virtual Environment Injection & Token-Passing (`src/load_python.rs`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`src/load_python.rs`](../../../src/load_python.rs) |
| **Document Path** | `reports/system-architecture/src/03_cpython_boot_and_venv_injection.md` |
| **Domain** | PyO3 FFI, CPython Environment & Engine Instantiation |
| **Target Persona** | Python & Systems Engineer (`site.addsitedir`, `sys.path`, `importlib`, CPython C-API, GIL tokens) |
| **Python Mental Anchor** | `site.addsitedir()`, `sys.path` manipulation, token-passing `'py` lifetime, `importlib.import_module()` |
| **Verification Mode** | Codebase Verified ([`src/load_python.rs`](../../../src/load_python.rs)) |
| **Confidence** | VERIFIED |

> **TL;DR:** `src/load_python.rs` initializes the embedded CPython 3.14 environment. It dynamically injects the dedicated `uv` virtual environment's site-packages via `site.addsitedir()`, imports the `ColdWindEngine` class, and instantiates the engine using a strict Token-Passing (`Python<'py>`) pattern.

---

## 🏛️ 1. The Embedded CPython Discovery Problem

When a native Rust binary embeds CPython via PyO3, the embedded interpreter defaults to the system Python's standard library. It has **no awareness** of third-party packages installed in your project's `.venv` (such as LangGraph, Pydantic, or OpenAI SDK).

If you attempt `py.import("coldwind.core")` without configuring the environment, CPython immediately raises `ModuleNotFoundError: No module named 'coldwind'`.

```text
+-----------------------------------------------------------------------------+
|                      CPYTHON VIRTUAL ENVIRONMENT INJECTION                  |
+-----------------------------------------------------------------------------+

 1. Compile Time (Cargo.toml / .env)
    COLDWIND_VENV = "<repo-root>/ai-workflow-task-agent/.venv"
         │
         │ Baked in via env!("COLDWIND_VENV")
         ▼
 2. Runtime Execution (gc_connect(py))
    +-------------------------------------------------------------------------+
    | Calls: site.addsitedir(                                                 |
    |   "<coldwind-venv-root>/lib/python3.14/site-packages"                   |
    | )                                                                       |
    +------------------------------------+------------------------------------+
                                         │
                                         ▼
 3. CPython 3.14 Internal State
    • Injects path into sys.path
    • Evaluates all *.pth files (enabling editable pip/uv installs)
    • Updates sys.modules search index
                                         │
                                         ▼
 4. Engine Instantiation (load_core & turn_on_boot)
    • py.import("coldwind.core.engine.coldwind_engine")  -> SUCCESS
    • engine_class.call0()?.call_method0("boot")        -> ONLINE
```

### The Python ↔ Rust Systems Mapping

| Python Standard Library Concept | Rust `src/load_python.rs` Reality | Systems Advantage |
| :--- | :--- | :--- |
| `site.addsitedir(path)` | `site.call_method1("addsitedir", (site_pak_location,))?` | Processes `.pth` files and directory packages correctly, unlike primitive `sys.path.append()`. |
| `os.environ["VIRTUAL_ENV"]` | `const VENV_LOCATION: &str = env!("COLDWIND_VENV")` | Compile-time constant check guarantees binary fails to build if `.venv` is missing. |
| Scoped `with gil:` block | `gc_connect(py: Python<'_>)` | **Token-Passing Pattern:** Reuses an existing `'py` token rather than re-acquiring the GIL. |
| `importlib.import_module("coldwind...")` | `py.import(core_mod_pth)?` | Resolves module directly from CPython's internal `sys.modules` dictionary. |

---

## 🔍 2. Keyword & Systems Decoder

| Keyword / Type | CPython / C-API Reality | Plain-English Systems Explanation |
| :--- | :--- | :--- |
| `env!("COLDWIND_VENV")` | Cargo compile-time macro | Inspects the environment at build time and embeds the string directly into the binary's `.rodata` segment. |
| `Python<'py>` | GIL Token (Capability type) | A zero-sized token proving that the current thread currently holds the CPython Global Interpreter Lock. |
| `Bound<'py, PyAny>` | Local `PyObject*` with lifetime | A high-performance smart pointer bound to lifetime `'py`. Automatically decrements the reference count on scope exit. |
| `site.addsitedir` | Python `site` module C hook | Adds a directory to `sys.path` AND parses `.pth` files (critical for editable package development). |
| `call0()` / `call_method0(...)` | `PyObject_CallNoArgs(...)` | CPython C-API call invoking a callable object with zero arguments without tuple allocation. |

---

## 🧱 3. Code Walkthrough

### 3.1 `gc_connect`: Virtual Environment Hook

```rust
const VENV_LOCATION: &str = env!("COLDWIND_VENV");

pub fn gc_connect(py: Python<'_>) -> PyResult<()> {
    let site = py.import("site")?;
    let site_pak_location = format!("{VENV_LOCATION}/lib/python3.14/site-packages");
    site.call_method1("addsitedir", (site_pak_location,))?;
    Ok(())
}
```

#### Why `site.addsitedir` Beats `sys.path.insert(0, ...)`

Many developers write:
```python
import sys
sys.path.insert(0, "/path/to/site-packages")
```
In modern Python 3.14 workspaces managed by `uv` or `pip`, packages are frequently installed in **editable mode** (`pip install -e .`), which creates `.pth` files (e.g. `__editable___coldwind_0_1_0_finder.pth`). 

A simple `sys.path.insert()` does **not** process `.pth` files. Only **`site.addsitedir()`** executes `.pth` discovery and registers import hooks correctly.

---

### 3.2 The Token-Passing Lifetime Pattern

Notice that none of the functions in `src/load_python.rs` call `Python::attach`:

```rust
pub fn load_core<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let core_mod_pth = "coldwind.core.engine.coldwind_engine";
    let engine_mod = py.import(core_mod_pth)?;
    let cold_wind_engine_c = engine_mod.getattr("ColdWindEngine")?;
    Ok(cold_wind_engine_c)
}
```

```text
  ANTI-PATTERN (Re-attaching GIL on every function):
  main() -> Python::attach -> load_python() -> Python::attach (Re-entrant GIL overhead!)

  TOKEN-PASSING PATTERN (src/load_python.rs):
  Python::attach(|py| {
      gc_connect(py)?;               // Passes 'py token
      let cls = load_core(py)?;      // Passes 'py token, returns Bound<'py, PyAny>
      let inst = turn_on_boot(py, &cls)?; // Passes 'py token
  })                                 // Single GIL scope, 0ms overhead!
```

By accepting `py: Python<'py>`, the caller in `src/main.rs` (Thread 2) acquires the GIL once at boot and passes the token down. This guarantees:
1. Zero re-entrant GIL acquisition overhead.
2. All returned `Bound<'py, PyAny>` references share the identical lifetime `'py`.
3. If any step fails with a Python exception, `?` bubbles the error cleanly.

---

### 3.3 Engine Instantiation & Booting

```rust
pub fn turn_on_boot<'py>(
    _py: Python<'py>,
    engine_class: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let engine_inst = engine_class.call0()?;
    engine_inst.call_method0("boot")?;
    Ok(engine_inst)
}
```

- **`engine_class.call0()?`:** Equivalent to `ColdWindEngine()` in Python. Invokes the `__init__` constructor and allocates the instance on the Python heap.
- **`engine_inst.call_method0("boot")?`:** Executes `ColdWindEngine.boot()`, initializing LangGraph node registries, LLM clients, and MCP tool managers.

---

## ⚠️ 4. Common Traps & CPython Pitfalls

### Trap 1: Hardcoding `python3.14` in Dynamic Environments
- **The Trap:** If the local virtual environment is upgraded from Python 3.13 to 3.14, a hardcoded path will point to a non-existent folder.
- **The Safeguard:** The build script and Cargo check verify `COLDWIND_VENV` and Python version alignment during compilation.

### Trap 2: Returning `Bound<'py, PyAny>` Across Thread Boundaries
- **The Rust Trap:** Trying to store `Bound<'py, PyAny>` in a struct that crosses threads.
- **The Compiler Error:** `Bound` contains a lifetime `'py` that is tied to the current thread's GIL token (`!Send`).
- **The Fix:** Call `.unbind()` to convert `Bound<'py, T>` into `Py<T>`, which is a reference-counted handle that can be safely stored in `SlintRuntime` across turns.

---

## ❓ 5. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | What happens if `COLDWIND_VENV` environment variable is not set at compile time? | `env!("COLDWIND_VENV")` causes `rustc` to emit a compile-time error, preventing broken binaries from compiling. | 2026-11-04 |
| Q2 | Does `site.addsitedir` duplicate paths if called multiple times? | No. CPython's `site.addsitedir` checks `sys.path` and avoids inserting duplicate directory paths. | 2026-11-04 |
| Q3 | Why doesn't `load_core` instantiate the engine directly? | Separating `load_core` (class loading) from `turn_on_boot` (instantiation) allows unit tests in `tests/test_load_core_boot.rs` to verify imports without triggering heavy LLM client initialization. | 2026-11-04 |
| Q4 | Which thread executes `gc_connect` and `load_core`? | Worker Thread 2 executes CPython virtual environment injection and engine boot, ensuring Thread 0 UI remains fully responsive. | 2026-11-15 |
