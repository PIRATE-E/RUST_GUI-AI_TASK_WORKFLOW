<div align="center">

<h1>🛡️ Code of Conduct &amp; Core Engineering Commandments</h1>

<p>
  <img src="https://img.shields.io/badge/Standard-Contributor_Covenant_2.1-2563EB?style=for-the-badge" alt="Contributor Covenant">
  <img src="https://img.shields.io/badge/Architecture-Hard_Invariants-DC2626?style=for-the-badge" alt="Hard Invariants">
  <img src="https://img.shields.io/badge/Domain_Law-AGENTS.md_Enforced-7C3AED?style=for-the-badge" alt="AGENTS.md">
</p>

<p>
  <em>Behavioral standards for collaborators AND non-negotiable Rust, PyO3 0.26, and Slint 1.16 architectural invariants that must never be violated in this codebase.</em>
</p>

</div>

---

## 🤝 Part 1: Community & Collaboration Pledge

We are committed to maintaining an open, rigorous, and welcoming engineering environment for all contributors, reviewers, and learners.

* **Constructive Technical Rigor:** Critique code, memory layouts, and architectural trade-offs—never people.
* **Zero Silent Regressions:** Never delete existing documentation depth, tests, or educational comments to save time. Every change must preserve or improve system clarity.
* **Truth Over Guesswork:** Never document unimplemented features as shipped, and never commit code without verifying invariants against `cargo test`.

---

## ⚙️ Part 2: The 4 PyO3 & Rust FFI Architectural Commandments

> [!CAUTION]
> **Why These Rules Live Here:** Bridging native Rust memory with CPython's garbage-collected heap introduces subtle lifetime, GIL, and re-entrancy traps that developers frequently forget. Violating any commandment below causes compile failures or runtime panics.

### Commandment 1: `Bound<'py, T>` vs `Py<T>` (Scoped Handle vs Heap Handle)

In PyO3 `0.26+` (PEP 703 compliant), Python object handles are split into two strict ownership types:

| Handle Type | Lifetime & Thread Safety | CPython Reality | When to Use |
| :--- | :--- | :--- | :--- |
| **`Bound<'py, T>`** | Tied to active GIL token `'py` (`!Send`, `!Sync`) | Smart pointer borrowing a `PyObject*` within a single `Python::attach(\|py\| ...)` block; skips redundant `Py_INCREF` overhead. | Local variable manipulation, method calls (`call_method1`), and argument extraction inside a GIL closure. |
| **`Py<T>`** | `'static` lifetime (`Send + Sync`) | Heap-allocated `PyObject*` owning a strong reference count independent of any stack scope. | Storing long-lived Python objects (`engine`, `context`, `session_state`, `ticket_in_cls`) inside `SlintRuntime` across turns. |

* **Conversion Rule:** Call `.unbind()` to store a `Bound<'py, T>` into a struct (`Py<T>`), and call `.bind(py)` inside `Python::attach(|py| ...)` to use it again.

---

### Commandment 2: Strict 3-Thread OS Isolation & GIL Confinement

`ai-workflow-gui` enforces a **Shared-Nothing 3-Thread Actor Topology** (`src/main.rs`):

1. **`Thread 0` (Slint UI Event Loop — `!Send`, Zero GIL):**
   * Owns `AppWindow`, GPU surfaces, `Rc<VecModel<ChatMessage>>`, and the `144 Hz` (`6.94 ms`) splash timer.
   * **FORBIDDEN:** `Thread 0` must **NEVER** call `Python::attach` or block on I/O channels.
2. **`Thread 2` (CPython 3.14 Worker — Owns GIL):**
   * Receives `WorkerState::CurrentUserMsg` over `mpsc::sync_channel(1)`, acquires `Python::attach(|py| ...)`, and executes `SlintRuntime::process_ticket()`.
   * **FORBIDDEN:** `Thread 2` must **NEVER** hold a direct `AppWindow` handle.
3. **`Thread 3` (Telemetry & ACID Disk Drainer — Zero GIL):**
   * Drains `GLOBAL_INJECTOR_LOG_ENTRY` (`sync_channel(1024)`) continuously, updates `TicketUiState`, writes JSONL lines to `<ticket_id>.swap` via `SwapLogFileWriter`, and dispatches UI snapshots to `Thread 0`.

---

### Commandment 3: `&self` Sinks vs `&mut self` Re-Entrancy Panic

When Python invokes a Rust `#[pyclass]` method or `#[pyfunction]` trampoline during a turn:

* **Always use `&self` (shared immutable borrow)** with interior synchronization (`mpsc::SyncSender`, `OnceLock`, `Atomic*`) on callbacks exposed to Python.
* **Never use `&mut self` on re-entrant callbacks:** If Rust calls into Python (`engine.process_ticket(py, ...)`) while holding a mutable borrow on a `PyCell<T>`, and Python calls back into Rust on the same object, PyO3 triggers a runtime `PyBorrowMutError` panic.

---

### Commandment 4: `slint::Weak<AppWindow>` UI Marshalling

Slint's `AppWindow` wraps thread-local `Rc` pointers (`!Send`) and cannot cross thread boundaries:

* **Before spawning a worker thread:** Call `let ui_weak = app.as_weak();` (`slint::Weak<AppWindow>` IS `Send + Sync`).
* **Inside `Thread 2` or `Thread 3`:** Schedule UI mutations onto `Thread 0` via `slint::invoke_from_event_loop(move || { ... })` and safely upgrade with `if let Some(app_ui) = ui_weak.upgrade() { ... }`.

---

## 📱 Part 3: The Android Studio ↔ Slint/Rust Mental Bridge

For developers coming from **Android Studio (XML layouts + Java/Kotlin)**, use this exact Rosetta Stone when working inside `ui/**/*.slint`:

| Concept | Android Studio (XML / Kotlin) | Slint + Rust (`ai-workflow-gui`) | Architectural Invariant |
| :--- | :--- | :--- | :--- |
| **Root Activity & Layout** | `MainActivity.kt` + `activity_main.xml` | `export component AppWindow inherits Window` | Combines layout tree, state enum (`AppState`), and Rust callbacks in one declarative unit. |
| **Type-Safe Binding** | `ViewBinding` / `DataBinding` | `slint::include_modules!()` (`build.rs`) | Compiles `ui/types.slint` structs (`ChatMessage`, `DrawerLogItem`) into zero-copy Rust structs in `$OUT_DIR`. |
| **RecyclerView & Adapter** | `RecyclerView.Adapter` + `DiffUtil` | `ListView` + `Rc<VecModel<ChatMessage>>` | Mutate active rows in place via `chat_model.set_row_data(idx, updated_msg)`—never rebuild the whole list. |
| **Polymorphic ViewHolder** | `getItemViewType(position)` | Conditional `if msg.is-user:` inside `VerticalLayout` | `inherits VerticalLayout` automatically measures dynamic row heights for `UserBubble` (`340px`) and `AiBubble` (`430px`). |
| **Background-to-UI Thread** | `Handler(Looper.getMainLooper()).post { ... }` | `slint::invoke_from_event_loop(move \|\| { ... })` | Marshals telemetry updates from `Thread 3` onto `Thread 0`'s event loop safely. |
| **Modal Bottom Sheet / Drawer** | `BottomSheetDialogFragment` + window scrim | `TicketDrawer` (`width: root.width * 0.75`) | Translates `x` smoothly (`180ms` cubic-bezier) with a backdrop `TouchArea` dismissal scrim. |

---

## 🧭 Part 4: Constitutional Domain Separation Law (`AGENTS.md`)

> [!NOTE]
> **Strict Analogical Hygiene:** Every code file and report in this repository obeys the Domain Separation Mandate defined in [`AGENTS.md`](./AGENTS.md):
> 1. **UI Layer (`ui/**/*.slint`):** Use **Android Studio** mental bridges (`XML`, `ViewBinding`, `RecyclerView`, `Looper`).
> 2. **Rust Core & FFI Layer (`src/**/*.rs`):** Use **CPython 3.14 internals** (`threading`, `queue.Queue`, `PyTypeObject`, `site.addsitedir`, `os.fsync`). **Never** mix Android/Java analogies into pure Rust backend files, and **never** use React/HTML DOM analogies anywhere.