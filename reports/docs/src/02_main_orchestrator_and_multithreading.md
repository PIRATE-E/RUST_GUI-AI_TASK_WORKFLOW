# 🧵 Concurrency Architecture, Thread Actor Loops & Main Orchestration (`src/main.rs`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`src/main.rs`](file:///home/pirate/development/ai-workflow-gui/src/main.rs) |
| **Document Path** | `reports/docs/src/02_main_orchestrator_and_multithreading.md` |
| **Domain** | Concurrency, Actor Pattern & Application Lifecycle |
| **Target Persona** | Python & Systems Engineer (`threading`, `queue.Queue`, Actor Pattern, Shared-Nothing Concurrency) |
| **Python Mental Anchor** | Python `threading.Thread`, `queue.Queue(maxsize=1)` actor loop, `sys.monitoring`, zero shared mutable state |
| **Verification Mode** | Codebase Verified ([`src/main.rs`](file:///home/pirate/development/ai-workflow-gui/src/main.rs)) |
| **Confidence** | VERIFIED |

> **TL;DR:** `src/main.rs` orchestrates the desktop application across 4 isolated threads using a pure Actor/Message-Passing model. Thread 0 renders the 60 FPS GUI, Thread 1 hosts the Python engine actor on a bounded `sync_channel(1)`, Thread 2 runs the Python logging hook, and Thread 3 aggregates real-time telemetry into `TicketUiState` with zero mutex locks.

---

## 🏛️ 1. Multi-Thread Concurrency Topology

In a standard Python desktop application (like PyQt or Tkinter), running heavy AI workloads on the main thread causes the dreaded "Application Not Responding" (beachball / window freeze). In Rust + Slint, UI components are strictly `!Send` (tied to Thread 0), while CPython 3.14 requires exclusive control of its GIL tokens.

`src/main.rs` solves this with a **4-Thread Shared-Nothing Architecture**:

```text
+-----------------------------------------------------------------------------+
|                      4-THREAD CONCURRENCY TOPOLOGY                          |
+-----------------------------------------------------------------------------+

 [ THREAD 0: UI EVENT LOOP ]                     [ THREAD 1: PYTHON WORKER ]
  • Owns AppWindow & GPU Renderer                 • Holds SlintRuntime alive
  • Owns Rc<VecModel<ChatMessage>>                • Bounded sync_channel(1) Actor
  • Generates Uuid::new_v4() ticket_id            • Acquires Python::attach GIL
  • Renders at 60 FPS                             • Executes process_ticket()
           │                                               │
           │ state_sender.send(CurrentUserMsg)             │
           └──────────────────────────────────────────────►│
                                                           │
 [ THREAD 2: CPYTHON ENGINE ]                              │ slint::invoke_from_event_loop
  • Runs LangGraph agent execution                         │ (Pushes TicketOutResult)
  • Fires rust_handle_overrite(log)                        ▼
           │                                      [ THREAD 0: UI UPDATE ]
           │ get_global_injector_self().emit()     • Mutates last_ai_msg in-place
           ▼                                       • Sets is_generating = false
 [ THREAD 3: TELEMETRY SINK ]
  • Private TicketUiState (Stack allocated)
  • Consumes log_receiver.recv()
  • Classifies AgentPhase & status quotes
  • Marshals live snapshot to Thread 0 via invoke_from_event_loop
```

### The Python ↔ Rust Concurrency Mapping

| Python Standard Library Pattern | Rust `src/main.rs` Architecture | Concurrency Advantage |
| :--- | :--- | :--- |
| `threading.Thread(target=worker, daemon=True)` | `std::thread::spawn(move \|\| { ... })` | Native OS thread spawned directly without Python GIL contention. |
| `queue.Queue(maxsize=1)` (Task pipeline) | `mpsc::sync_channel::<WorkerState>(1)` | Guarantees backpressure: only 1 in-flight ticket turn at any second. |
| `threading.Lock()` protecting shared state | **Zero Locks!** (Thread Confinement) | `TicketUiState` is private to Thread 3. No mutex deadlocks or lock poisoning. |
| `queue.Queue(maxsize=1024)` (Log queue) | `init_sync_send_all()` channel | High-speed bounded buffer draining execution logs asynchronously. |
| GUI thread dispatcher: `root.after(0, fn)` | `slint::invoke_from_event_loop(move \|\| ...)` | Thread-safe closure queuing onto Thread 0's native event loop. |

---

## 🔍 2. Keyword & Systems Decoder

| Rust Keyword / Construct | Systems / Python Analog | Plain-English Explanation |
| :--- | :--- | :--- |
| `WorkerState` (enum) | Typed message payload / Action dict | An algebraic data type representing commands sent to the worker thread (`CurrentUserMsg` or `Destroy`). |
| `mpsc::sync_channel(1)` | `queue.Queue(maxsize=1)` | Synchronous bounded channel. Sender blocks if a message is already pending, enforcing strict turn serialization. |
| `app.as_weak()` | `weakref.ref(app)` | Creates a non-owning `Weak<AppWindow>` reference that can cross thread boundaries safely (`Send`). |
| `ui_weak.upgrade()` | Dereferencing a weakref | Checks if `AppWindow` is still alive on Thread 0. Returns `Some(app)` if alive, or `None` if the window was closed. |
| `ModelRc<T>` | Reactive list adapter handle | A smart pointer wrapping a dynamic list model. When modified, Slint marks affected rows dirty for GPU redraw. |

---

## 🧱 3. The 4 Concurrent Threads in Detail

### 3.1 Thread 0: Main UI Thread & Event Loop

- **Lifecycle:** Created when `main()` executes `let app = AppWindow::new().unwrap();` and blocks inside `app.run().unwrap()`.
- **Primary Duties:**
  1. Initializes `AppState::Booting` to display the splash screen.
  2. Owns `message_stack = Rc::new(VecModel::<ChatMessage>::default())`.
  3. Stamps the canonical ticket identity when user clicks Send:
     ```rust
     turn_counter += 1;
     let ticket_id = format!("development-{}", Uuid::new_v4());
     ```
  4. Appends `UserBubble` and an initial placeholder `AiBubble` (`is_generating: true`) to `message_stack`.
  5. Dispatches `WorkerState::CurrentUserMsg` to Thread 1.

---

### 3.2 Thread 1: Python Engine Worker Actor Loop

- **Lifecycle:** Spawned at line 60. Runs continuously until the application terminates.
- **The Actor Pattern:** Listens on `state_receiver.recv()`:
  ```rust
  while let Ok(worker_state) = state_receiver.recv() {
      match worker_state {
          WorkerState::CurrentUserMsg { prompt, ticket_id, turn_counter } => {
              Python::attach(|py| {
                  let ticket_out = slint_runtime_obj.process_ticket(
                      py, &prompt, &ticket_id, turn_counter, 60
                  )?;
                  // Marshal completion back to Thread 0
              });
          }
          WorkerState::Destroy => break,
      }
  }
  ```
- **Why `SlintRuntime` Lives Here:** `SlintRuntime` holds `Py<PyAny>` handles to the CPython engine. By confining `SlintRuntime` to Thread 1, CPython is never accessed concurrently from multiple threads, completely eliminating GIL deadlocks.

---

### 3.3 Thread 2: Python Logging Hook (In-Engine)

- **Lifecycle:** Runs within the execution context of the Python agent workflow whenever LangGraph executes a node or an LLM call finishes.
- **Mechanism:** Injected via descriptor monkey-patching in [`src/log_entry_handler.rs`](file:///home/pirate/development/ai-workflow-gui/src/log_entry_handler.rs). It intercepts `RustGuiEventHandler.handle(log)` and pushes the deserialized `LogEntry` into `GLOBAL_INJECTOR_LOG_ENTRY`.

---

### 3.4 Thread 3: Background Telemetry Consumer (Thread-Confinement)

- **Lifecycle:** Spawned at line 178. Listens on `log_receiver.recv()`:
  ```rust
  let mut state = TicketUiState::new();
  while let Ok(log) = log_receiver.recv() {
      state.parse_log(&log);

      let task_progress = state.task_progress();
      let rpm_count = state.rpm_count;
      let total_requests = state.total_requests;
      let fail_count = state.failed_count;
      let headline = state.current_headline.clone();

      slint::invoke_from_event_loop(move || {
          // Mutate Thread 0 last_ai_msg in-place!
      });
  }
  ```
- **The Thread-Confinement Principle:** Notice that `state` is a plain `mut TicketUiState` allocated on Thread 3's stack. It is **not** wrapped in `Arc<Mutex<TicketUiState>>`. Thread 3 extracts plain value snapshots (integers and strings) and passes them by value to `invoke_from_event_loop`.

---

## ⚡ 4. In-Place `VecModel` Bubble Mutation

When Thread 3 receives a telemetry update or Thread 1 completes a ticket turn, they must update the active AI chat bubble on Thread 0.

Notice how lines 201–222 perform in-place mutation:

```rust
let chat_model = app_ui.get_messages();
let count = chat_model.row_count();
if count > 0 {
    let last_idx = count - 1;
    if let Some(mut last_ai_chat_msg) = chat_model.row_data(last_idx) {
        if last_ai_chat_msg.is_generating {
            // Update telemetry fields:
            last_ai_chat_msg.task_progress = task_progress.into();
            last_ai_chat_msg.rpm_count = rpm_count;
            last_ai_chat_msg.api_count = total_requests;
            last_ai_chat_msg.fail_count = fail_count;
            last_ai_chat_msg.live_summary = headline.into();

            // Push updated struct back to exact row index:
            chat_model.set_row_data(last_idx, last_ai_chat_msg.clone());

            // Synchronize active drawer if user has it open:
            if app_ui.get_drawer_open() && app_ui.get_active_ticket().is_generating {
                app_ui.set_active_ticket(last_ai_chat_msg);
            }
        }
    }
}
```

```text
VecModel<ChatMessage> on Thread 0
+----------------------------------------------------+
| Row 0: UserBubble ("Analyze database logs")        |
| Row 1: AiBubble   ("Previous answer...")           |
| Row 2: UserBubble ("What is the latency?")         |
| Row 3: AiBubble   (Active: is_generating == true)  | ◄── chat_model.set_row_data(3, updated)
+----------------------------------------------------+
```

### Why In-Place Mutation Wins

1. **Zero List Rebuilding:** Only Row 3 is marked dirty. Rows 0, 1, and 2 remain untouched on the GPU.
2. **Scroll Lock Preservation:** Appending and deleting dummy rows causes the scrollbar to bounce. In-place mutation maintains stable scroll position.
3. **Primary Key Protection:** The `ticket_id` assigned by Thread 0 is preserved intact, preventing split-brain ticket lookups.

---

## ⚠️ 5. Common Traps & Concurrency Pitfalls

### Trap 1: Crossing Threads with `Rc<VecModel>`
- **The Rust Trap:** Trying to pass `message_stack.clone()` into `std::thread::spawn`.
- **The Compiler Error:** `error[E0277]: Rc<VecModel<ChatMessage>> cannot be sent between threads safely`.
- **The Fix:** `Rc` is strictly `!Send`. Only `app.as_weak()` (which is `Send`) can cross thread boundaries. Inside `invoke_from_event_loop`, call `app_ui.get_messages()` on Thread 0.

### Trap 2: Invoking `Python::attach` on Thread 0
- **The GIL Trap:** Calling Python code directly inside `app.on_send_message`.
- **The Failure:** If Python acquires the GIL and performs network I/O, Thread 0 freezes completely.
- **The Fix:** Thread 0 sends a message across `sync_channel(1)` to Thread 1. Thread 1 owns all GIL operations.

---

## ❓ 6. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | What happens if the user clicks Send while a workflow is already running? | The UI input box is disabled or backpressured because `state_sender` has a capacity of 1, preventing overlapping turns from corrupting the CPython working state. | 2026-11-04 |
| Q2 | Why does Thread 3 check `if last_ai_chat_msg.is_generating` before updating? | To ensure that delayed logs arriving after a turn finishes never overwrite a completed transcript or reset finalized metrics. | 2026-11-04 |
| Q3 | How does the drawer stay synchronized with the active bubble? | If `drawer_open` is true, Thread 3 calls `app_ui.set_active_ticket(last_ai_chat_msg)` at 60 FPS in the same `invoke_from_event_loop` dispatch. | 2026-11-04 |
