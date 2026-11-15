# ❄️ Slint GUI vs Desktop CLI: Architecture Gap, Telemetry Sinks & Modern Desktop UX Report

| Field | Value |
| :--- | :--- |
| **Document ID** | `SLINT-GUI-ISSUE-01` |
| **Status** | Active Architectural Blueprint & Gap Analysis |
| **Type** | Investigation / Architecture Gap Analysis |
| **Target Domain** | `reports/parity-audits/` (Migrated from legacy urgent development domain) |
| **Author** | Cold Wind Engineering Pair |
| **Date** | 2026-10-31 |
| **Confidence** | VERIFIED (Audited against live `src/` and `coldwind-desktop` code) |

> **TL;DR:** While the reference Desktop CLI features an interactive telemetry loop, native command parsing, and live spinners, the Rust + Slint GUI (`ai-workflow-gui`) contains 7 critical architectural hazards—including an undrained `sync_channel(1024)` log sink causing worker thread deadlock, UI thread freezes on multi-clicks, and silent failure masking. This document establishes the diagnostic audit and four-pillar architecture to bring the GUI to full desktop parity.

---

## 1. Executive Summary

The **Cold Wind AI** system has completed its headless decoupling phase: `ColdWindEngine` operates as a stateless turnkey facade executing discrete conversational turns via `process_ticket(TicketIn) -> TicketOut`. 

While the reference **Desktop CLI** (`coldwind-desktop`) features a fully functional Rich-based interactive telemetry loop (`DesktopStatusHandler`, TCP socket debugging, multi-argument slash command execution, and rate-limiting spinners), the native **Rust + Slint Desktop GUI** (`ai-workflow-gui`) currently operates at a baseline Phase 0.0.1 proof-of-concept.

This report conducts a forensic, component-by-component architectural audit comparing the Desktop CLI against the Slint GUI implementation. It identifies **7 structural gaps and operational hazards**, including an unconsumed bounded log channel (`sync_channel(1024)`) creating an imminent deadlock risk, incomplete slash command dispatching (`/clear`, `/exit`), unhandled turn failures in the UI model, missing tool argument inspection, and thread-safety hazards during turn execution. Finally, it outlines an industrial-grade **Gemini-style collapsible thinking accordion**, non-blocking UI locks, and a native Rust command execution pipeline to bring the Slint GUI to parity with modern desktop AI applications.

### Audit Findings & Strategic Remediation Summary

| Gap ID | Problem Area | Root Mechanism | Severity | Remediation Pillar |
| :---: | :--- | :--- | :---: | :--- |
| **GAP-01** | Bounded Log Channel Deadlock | `sync_channel(1024)` never drained; blocks on 1025th log | 🔴 **Critical** | **Pillar 1:** Dedicated Thread L Telemetry Drainer |
| **GAP-02** | Stubbed Slash Commands | `/clear` and `/exit` fast-paths commented out | 🟡 **Major** | **Pillar 2:** Native Rust `ClientCommand` Parser |
| **GAP-03** | Silent Failure Masking | Failed `TicketOut` yields empty transcript string | 🔴 **Critical** | **Pillar 2:** Dedicated Red Error Diagnostics Bubble |
| **GAP-04** | UI Thread 0 Freezing | Blocking `sync_channel(1)::send` locks event loop on double click | 🔴 **Critical** | **Pillar 3:** Non-blocking `try_send` + `is-processing` lock |
| **GAP-05** | Invisible Tool Registry | `TOOL_INFO_LIST` cached in memory but absent from UI | 🔵 **Gap** | **Pillar 4:** Sliding Tools Drawer Component |
| **GAP-06** | Stubbed Exception Handler | `SlintExceptionHandler` drops exceptions with empty body | 🟡 **Major** | **Pillar 1:** Structured Diagnostic Telemetry Ingestion |
| **GAP-07** | List Direction Inversion | `vec_model.push` appends downward; header says newest on top | 🟡 **Major** | **Pillar 4:** Bottom-Pinned ListView with Auto-Scroll |

---

## 2. System Context & Interdependence Map

The Cold Wind architecture isolates presentation platforms from the Python agent brain. Platforms connect as ambassadors driving the turnkey engine.

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                PRESENTATION PLATFORMS                                  │
│                                                                                        │
│   ┌─────────────────────────────────────────┐   ┌──────────────────────────────────┐   │
│   │       Desktop CLI (Reference Impl)      │   │   Slint GUI (ai-workflow-gui)    │   │
│   │  • Rich status spinner & live RPM countdown│   │  • Slint 1.16 reactive desktop UI│   │
│   │  • TCP Port 59700 log streaming server  │   │  • PyO3 0.26 embedded CPython    │   │
│   │  • OnRunTimeRegistry Slash Commands     │   │  • Dual-thread MPSC actor loop   │   │
│   └────────────────────┬────────────────────┘   └─────────────────┬────────────────┘   │
└────────────────────────┼──────────────────────────────────────────┼────────────────────┘
                         │ 1. activate_context(context)             │
                         │ 2. engine.boot()                         │
                         │ 3. engine.process_ticket(TicketIn)       │
                         ▼                                          ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              COLDWIND CORE ENGINE FACADE                               │
│                                 (class ColdWindEngine)                                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  • Compiles LangGraph StateGraph (Router, ClassifyAgent, ToolSelector, AgentModeNode)  │
│  • Dispatches structured LogEntry events via OnTimeRegistry -> Handler sinks           │
│  • Executes discrete single-turn ticket: TicketIn -> TicketOut                         │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Comparative Component Matrix

| Subsystem / Responsibility | Desktop CLI (`coldwind-desktop`) | Slint GUI (`ai-workflow-gui`) | Status / Severity |
| :--- | :--- | :--- | :--- |
| **Telemetry Interception** | `DesktopStatusHandler` (Rich spinner) & `DashBoardHandler` (TCP socket) | `RustGuiEventHandler` forwards to `sync_channel(1024)` | 🔴 **Critical:** Log queue is never drained; deadlocks at 1024 items |
| **User Activity Display** | Live single-line status: `[Task X/Y: Activity] \| API: N/40 RPM` | Static header text: `🟢 Online`; no activity display | 🟡 **Major Gap:** User is blind during 5–30s LLM execution |
| **Slash Command Parsing** | `ParseCommand` extracts commands and `--flag value` options | `SlintCommandParser` has temporary `split()` monkey-patch | 🟡 **Major Gap:** No `--options` parsing; bypasses native Rust |
| **Session Clear (`/clear`)** | Resets `session_state`, clears terminal, calls `StateAccessor` | Returns static transcript; `clear_session` commented out | 🟡 **Major Gap:** Python session state and UI stack are not reset |
| **Application Exit (`/exit`)** | Calls `context.request_exit()` and cleanly exits while loop | Returns static transcript; `WorkerState::Destroy` empty | 🟡 **Major Gap:** Window remains open; threads not joined |
| **Registered Tools View** | Prints Rich table with Name, Description, Arguments schema | Populates `TOOL_INFO_LIST` in Rust; zero UI display | 🔵 **Missing Feature:** No tools drawer or per-turn tool toggles |
| **Error & Traceback UX** | `RichTracebackManager` formats frames; prints bold red error | Returns `Ok(TicketOutResult)` on failure; blank AI bubble | 🔴 **Critical:** Failed tickets render as blank bubbles |
| **Turn Input Lock** | Blocking terminal prompt; captures `Ctrl+C` cleanly | `state_sender.send()` with no UI disable lock | 🔴 **Critical:** Multi-clicking Send freezes UI Thread 0 |

---

## 3. Detailed Forensic Analysis & Structural Gaps

### Forensic Matrix of Architectural Deficiencies

| Defect Identifier | File & Location | Concrete Hazard | Diagnostic Signature |
| :--- | :--- | :--- | :--- |
| **GAP-01: Log Deadlock** | [`src/lib.rs`](../../../src/lib.rs#L70-L75), [`src/main.rs`](../../../src/main.rs#L48) | Futex hang in Python worker thread | Complete application freeze on 1025th emitted log event |
| **GAP-02: Slash Command Stubs** | [`src/slint_py_engine.rs`](../../../src/slint_py_engine.rs#L115-L128) | Memory leak & unmanaged process state | Phantom "Cleared" bubbles; inability to close app cleanly |
| **GAP-03: Silent Failure Masking** | [`src/slint_py_engine.rs`](../../../src/slint_py_engine.rs#L145-L158), [`src/main.rs`](../../../src/main.rs#L85-L115) | Error information swallowed | AI response bubbles appear completely empty with 0 text |
| **GAP-04: UI Thread Lock** | [`src/main.rs`](../../../src/main.rs#L180-L195) | Main event loop blocked | Window manager displays "Application Not Responding" dialog |
| **GAP-05: Missing Tools View** | [`src/slint_py_engine.rs`](../../../src/slint_py_engine.rs#L40-L50) | Inaccessible tool metadata | User cannot see available tools or argument schemas |
| **GAP-06: Empty Exception Sink** | `coldwind/slint_gui/runtime/ui/exception_handler.py` | Diagnostic black hole | Uncaught background errors disappear silently |
| **GAP-07: Inverted List Order** | [`ui/app-window.slint`](../../../ui/app-window.slint#L12-L25), [`src/main.rs`](../../../src/main.rs#L85-L95) | Misaligned UX expectations | User must manually scroll to find new messages |

### Gap 1: In-Memory Bounded Log Sink Deadlock (`sync_channel(1024)`)

In [`src/lib.rs`](../../../src/lib.rs), the logging bridge initializes an MPSC channel with a hard capacity:
```rust
pub fn init_sync_send_all() -> ReceiverLogEntry {
    let (sender, receiver) = mpsc::sync_channel::<LogEntry>(1024);
    let _ = GLOBAL_INJECTOR_LOG_ENTRY.set(InjectorLogEntry(sender));
    ReceiverLogEntry(receiver)
}
```
In [`src/main.rs`](../../../src/main.rs), Thread N initializes this channel at startup:
```rust
let log_receiver = init_sync_send_all();
```

**The Failure Mechanism:**
Core emits between 10 and 50 `LogEntry` records per turn (across `ROUTER`, `CLASSIFIER`, `AGENT_WORKFLOW`, `TOOL_EXECUTION`, and `API_CALL`). Because `log_receiver` is an unused local variable inside the thread closure, **no thread ever calls `log_receiver.recv()` or `try_recv()`**.
Once the total lifetime log count reaches 1024, `mpsc::SyncSender::send()` blocks synchronously inside `InjectorLogEntry::emit()`. Because `emit()` is called directly inside Python's execution thread during `rust_handle_overrite`, **the Python worker thread permanently hangs in an unrecoverable kernel futex lock**.

```text
Core LangGraph Node
       │
       ▼
OnTimeRegistry.handle(LogEntry)
       │
       ▼
rust_handle_overrite(log_entry)
       │
       ▼
InjectorLogEntry.emit() ──► SyncSender::send(LogEntry)
                                   │
                                   ▼ [Capacity: 1024]
                     ┌───────────────────────────┐
                     │ [Msg 1] [Msg 2] ... [1024]│ ◄── Receiver NEVER reads!
                     └───────────────────────────┘
                                   │
                           (Msg 1025 Arrives)
                                   │
                                   ▼
                   💥 DEADLOCK: Worker Thread Frozen!
```

---

### Gap 2: Incomplete Slash Command Handshake & Dummy Fast-Paths

In Desktop CLI, slash commands are partitioned into two tiers:
1. **Host-Platform Commands (`/clear`, `/exit`):** Handled before submitting tickets to prevent unnecessary LLM invocations and reset local process state.
2. **Core Workflow Commands (`/llm`, `/agent`, `/tool`):** Dispatched via `CommandResult(data={"message_type": cmd})` to configure LangGraph router nodes.

In [`src/slint_py_engine.rs`](../../../src/slint_py_engine.rs), the fast-path interceptors contain disabled logic:
```rust
// Guard B: Fast-path intercept for exit
if matches!(user_message.to_lowercase().as_str(), "exit" | "quit" | "/exit") {
    // self.request_exit(py)?;  <-- COMMENTED OUT!
    return Ok(TicketOutResult { ... is_exiting: true });
}

// Guard C: Fast-path intercept for /clear
if user_message.eq_ignore_ascii_case("/clear") {
    // self.clear_session(py)?; <-- COMMENTED OUT!
    return Ok(TicketOutResult { ... });
}
```

**Architectural Consequences:**
- When the user types `/clear`, `self.session_state` on the Python heap is **not** cleared (`{"messages": [...]}` remains intact). Slint's `message_stack` on Thread 0 is **not** cleared. The user is presented with a fake bubble: `"Chat history cleared."` while memory persists.
- When the user types `/exit`, `WorkerState::Destroy` inside [`src/main.rs`](../../../src/main.rs) is an empty stub. The application does not quit, and models remain unreleased.
- In `SlintGuiContext.py`, `SlintCommandParser` lacks the `--options` parsing capability found in Desktop's `ParseCommand`. Submitting `/tool --name web_search query` fails to parse flags.

---

### Gap 3: Silent Failure Masking (`ticket_out.status == "FAILED"`)

When an execution error occurs in Core (such as an invalid API key, network timeout, or graph recursion limit), `ColdWindEngine.process_ticket()` returns a `TicketOut` object with:
- `status = "FAILED"`
- `transcript = ""`
- `error_info = {"error": "...", "traceback": "..."}`

In [`src/slint_py_engine.rs`](../../../src/slint_py_engine.rs):
```rust
if status == "COMPLETED" {
    // ...
} else {
    // Extracts error_msg but returns Ok(TicketOutResult):
    Ok(TicketOutResult {
        ticket_id,
        status, // "FAILED"
        transcript: String::new(), // EMPTY!
        latency_ms,
        error_message: Some(err_msg),
        is_exiting: false,
    })
}
```
In [`src/main.rs`](../../../src/main.rs):
```rust
match slint_runtime_obj.process_ticket(py, &prompt, 60) {
    Ok(ticket_out) => {
        // Pushes ChatMessage without checking ticket_out.status!
        let chat_message = ChatMessage {
            is_user: false,
            sender: "AI".into(),
            text: ticket_out.transcript.into(), // Pushes "" !
            timestamp: ticket_out.latency_ms.to_string().into(),
        };
        // vec_model.push(chat_message);
    }
}
```

**User Impact:** When a failure occurs, the GUI displays a completely blank AI message bubble with a timestamp. The actual error string (`error_message`) is discarded in native memory and never rendered.

---

### Gap 4: UI Thread Contention & Multi-Click Lock (`state_sender.send()`)

[`src/main.rs`](../../../src/main.rs) establishes communication between Thread 0 and Thread N via a synchronous channel with capacity 1:
```rust
let (state_sender, state_receiver) = mpsc::sync_channel::<WorkerState>(1);
```
In `app.on_send_message()`:
```rust
_ = state_sender.send(WorkerState::CurrentUserMsg(String::from(text.clone())));
```

**The Failure Mechanism:**
- While Core is executing an LLM turn on Thread N, the worker is busy inside `slint_runtime_obj.process_ticket(py, ...)`. The single slot of `state_sender` is empty until the user sends another message.
- If the user sends a message while the worker is busy, message 1 enters the buffer.
- If the user clicks **Send** again (or presses Enter twice), `state_sender.send(...)` blocks synchronously.
- Because `on_send_message` runs directly on **Thread 0 (Main UI Thread)**, blocking `send()` **instantly freezes the entire Slint window at 0 FPS**, crashing OS window responsiveness.

---

### Gap 5: Invisible Tool Registry (`TOOL_INFO_LIST`)

At startup, `SlintRuntime::boot()` iterates over `engine.tools` and caches tool names and descriptions into a global `OnceLock`:
```rust
pub static TOOL_INFO_LIST: OnceLock<Vec<ToolMetadata>> = OnceLock::new();
```
- In Desktop CLI, these tools are formatted into an ANSI table displaying `Name`, `Description`, and `Arguments` (inspected via `get_tool_argument_schema(tool)`).
- In Slint GUI, `TOOL_INFO_LIST` is loaded in memory but completely disconnected from [`ui/app-window.slint`](../../../ui/app-window.slint). The user has no visual mechanism to discover active MCP tools, view required schema parameters, or toggle tools on and off.

---

### Gap 6: Stubbed Exception Handler (`SlintExceptionHandler`)

In `coldwind/slint_gui/runtime/ui/exception_handler.py`:
```python
class SlintExceptionHandler:
    @classmethod
    def handle_exception(
        self,
        error: Exception,
        context: str = "",
        extra_context: dict[str, Any] | None = None,
    ):
        """
        decided we gonna make that emit the error log to the slint but the slint would gonna show those error logs different way !!
        """
        # COMPLETELY EMPTY!
```
When unhandled errors occur during background task execution, the handler discards them silently. No structured `LogEntry` with `LOG_LEVEL=ERROR` is dispatched, preventing the GUI from capturing diagnostic crash reports.

---

### Gap 7: Message Stack Direction & Auto-Scroll Inversion

In [`ui/app-window.slint`](../../../ui/app-window.slint), the chat header states:
```slint
Text { text: "💬 Chat Session (Newest on Top)"; }
```
However, [`src/main.rs`](../../../src/main.rs) appends incoming messages using:
```rust
vec_model.push(chat_message);
```
In Slint's `ListView`, `vec_model.push()` appends elements to the **end (bottom)** of the model, not the top. This creates an architectural contradiction:
1. The UI claims newest messages appear at the top.
2. Messages actually render chronologically downward.
3. The `ListView` lacks programmatic scroll-to-bottom mechanics, forcing the user to manually scroll down after every turn.

---

## 4. Proposed Architecture & Modern Desktop UX Upgrades

To bridge these gaps, the Slint GUI requires an upgraded architecture structured into four modular pillars:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        UPGRADED SLINT GUI ARCHITECTURE TOPOLOGY                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│   THREAD 0: SLINT UI LOOP (60 FPS GPU)                                                 │
│   ├── Reactive Properties: is-processing, current-task, api-rpm, failed-count         │
│   ├── UI Components: Collapsible Thinking Bubble, Mode Selector Pills, Tools Drawer    │
│   └── Safe Marshalling: Dispatches non-blocking try_send(CurrentUserMsg)               │
│                                                                                        │
│                              ▲                           │                             │
│       invoke_from_event_loop │                           │ mpsc::channel               │
│       (Plain Data Payload)   │                           │ (Non-blocking try_send)     │
│                              │                           ▼                             │
│                                                                                        │
│   THREAD N: ENGINE WORKER THREAD (CPython 3.14 GIL)                                    │
│   ├── Actor Loop: while let Ok(cmd) = state_receiver.recv()                            │
│   ├── Turnkey Dispatch: slint_runtime_obj.process_ticket(py, prompt, timeout)          │
│   └── Native Slash Commands: /clear (session reset) & /exit (clean shutdown)           │
│                                                                                        │
│                              │                                                         │
│                              │ mpsc::sync_channel (LogStream)                          │
│                              ▼                                                         │
│                                                                                        │
│   THREAD L: TELEMETRY DEMULTIPLEXER (Drainer Thread)                                   │
│   ├── Drains log_receiver in tight, zero-alloc loop (prevents 1024 deadlock)           │
│   ├── Aggregates [Task X/Y], RPM limits, and failure counts (DesktopStatusHandler)     │
│   └── Posts live updates to Thread 0 via slint::invoke_from_event_loop                 │
│                                                                                        │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Strategic Architectural Pillars

| Pillar | Subsystem Focus | Key Deliverables & Interfaces | Primary Invariant Enforced |
| :---: | :--- | :--- | :--- |
| **Pillar 1** | Telemetry Ingestion & Live Display | Thread L drainer thread, aggregated RPM status bar, Gemini-style thinking accordion | Bounded log channels must never block engine execution |
| **Pillar 2** | Native Slash Command Engine | Strong-typed `ClientCommand` Rust enum, zero-token fast-paths, diagnostic error bubble | Platform lifecycle actions must never consume cloud tokens |
| **Pillar 3** | Non-blocking Input & Thread Safety | `try_send` input dispatch, `is-processing` button disabling, 0 FPS stall guard | Thread 0 must remain unblocked at 60 FPS under all loads |
| **Pillar 4** | Tools Drawer & Modern Layout | Slide-out MCP tools drawer, bottom-pinned auto-scrolling `ListView`, dark WhatsApp theme | Presentation must provide full parity with CLI features |

---

### Pillar 1: Dedicated Telemetry Drainer & Collapsible "Thinking" Accordion

To prevent channel deadlocks and provide Gemini-grade visibility:
1. **Spawn Thread L (Telemetry Drainer):** A dedicated background thread continuously drains `ReceiverLogEntry`. It parses `LogCategory::AgentWorkflow` and `STATUS` messages, maintaining a local turn ledger.
2. **Aggregated Status Header:** Thread L forwards metric updates to Thread 0, updating top bar properties:
   ```slint
   in property <string> task-status: "[Task 1/3: Router]";
   in property <string> rpm-status: "API: 4/40 RPM";
   ```
3. **Collapsible Thinking Accordion in `ChatMessage`:**
   Each AI message bubble contains an embedded collapsible step view:
   ```slint
   export struct ChatMessage {
       text: string,
       is-user: bool,
       sender: string,
       timestamp: string,
       is-error: bool,
       thinking-steps: [string],
       thinking-duration: string,
       is-expanded: bool,
   }
   ```
   During execution, the bubble displays an animated shimmer: `⚡ Thinking... [Task 2/4: ToolExecution]`. Once completed, it collapses into: `▼ Thought for 2.1s (4 steps)`. The user clicks to expand and inspect intermediate node logs without cluttering the chat feed.

---

### Pillar 2: Native Rust Slash Command Parsing Engine

Replace the Python-side `split()` monkey-patch with a strongly-typed native Rust parsing module (`src/slash_commands.rs`):

```rust
pub enum ClientCommand {
    WorkflowRoute { mode: String, query: String },
    ClearSession,
    ExitApplication,
    ShowHelp,
    NormalPrompt(String),
}

impl ClientCommand {
    pub fn parse(input: &str) -> Self {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return ClientCommand::NormalPrompt(trimmed.to_string());
        }
        let mut parts = trimmed[1..].splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").to_lowercase();
        let rest = parts.next().unwrap_or("").trim().to_string();

        match cmd.as_str() {
            "clear" => ClientCommand::ClearSession,
            "exit" | "quit" => ClientCommand::ExitApplication,
            "help" => ClientCommand::ShowHelp,
            "llm" | "agent" | "tool" => ClientCommand::WorkflowRoute { mode: cmd, query: rest },
            _ => ClientCommand::NormalPrompt(trimmed.to_string()),
        }
    }
}
```

#### Execution Semantics:
- **`ClearSession`**: Thread 0 immediately clears `message_stack`. Thread N invokes `session_state = {"messages": [], "message_type": None}` and triggers `ToolResponseManager().clear_response()`. Zero LLM tokens consumed.
- **`ExitApplication`**: Sends `WorkerState::Destroy` $\rightarrow$ Thread N calls `ColdWindEngine.shutdown()` $\rightarrow$ Thread 0 calls `slint::quit_event_loop()`.
- **`WorkflowRoute`**: Maps directly to `TicketIn.user_message = query` with `working_state["message_type"] = mode`, forcing LangGraph router dispatch without relying on prompt classification.

---

### Pillar 3: UI Busy-Lock & Non-Blocking State Dispatch

To guarantee 60 FPS rendering under heavy user interactions:
1. **Declare Reactive State in Slint:**
   ```slint
   in-out property <bool> is-processing: false;
   ```
2. **Bind Input Elements:**
   ```slint
   send-btn := Button {
       text: root.is-processing ? "⏳" : "Send";
       enabled: !root.is-processing && root.current-input != "";
       clicked => { root.send-message(); }
   }
   input-box := LineEdit {
       enabled: !root.is-processing;
       accepted => { if (!root.is-processing) { root.send-message(); } }
   }
   ```
3. **Non-Blocking Dispatch on Thread 0:**
   In [`src/main.rs`](../../../src/main.rs), replace blocking `send()` with `try_send()`:
   ```rust
   match state_sender.try_send(WorkerState::CurrentUserMsg(prompt)) {
       Ok(_) => {
           app.set_is_processing(true);
           // Append user bubble to UI...
       }
       Err(mpsc::TrySendError::Full(_)) => {
           // Queue busy: reject input gracefully without locking Thread 0
       }
       Err(mpsc::TrySendError::Disconnected(_)) => {
           // Worker crashed: trigger error state
       }
   }
   ```

---

### Pillar 4: Registered Tools Slide-Out Drawer

Expose `TOOL_INFO_LIST` via a slide-out drawer or modal dialog in Slint:
1. **Slint Drawer Component (`ui/components/tools_drawer.slint`):**
   A sliding panel bound to `in property <[ToolInfo]> registered-tools;`.
2. **Tool Record Schema:**
   ```slint
   export struct ToolInfo {
       name: string,
       description: string,
       enabled: bool,
   }
   ```
3. **Live Filter Integration:**
   When the user toggles a tool off in the UI, Thread 0 updates an `active_tools` bitset or list. Thread N reads this list to filter `TicketIn.allowed_tools`, giving users direct control over tool availability per session.

---

## 5. Implementation Roadmap & Verification Plan

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              PHASED EXECUTION TIMELINE                                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│  PHASE 1: STABILITY & LIFECYCLE (Immediate Priority)                                  │
│  ├── Spawn Thread L to drain ReceiverLogEntry (Eliminates 1024 deadlock hazard)        │
│  ├── Implement is-processing lock & try_send() in main.rs (Prevents UI freezing)       │
│  └── Wire WorkerState::Destroy -> slint_runtime_obj.shutdown() -> quit_event_loop()    │
│                                                                                        │
│  PHASE 2: COMMANDS & SESSION INTEGRITY                                                 │
│  ├── Implement native ClientCommand parser in Rust                                     │
│  ├── Wire live /clear handler (resets session_state in Python and message_stack in UI) │
│  └── Render failed TicketOut responses as distinct Red Diagnostic Error Bubbles       │
│                                                                                        │
│  PHASE 3: GEMINI-STYLE TELEMETRY & THINKING DRAWER                                     │
│  ├── Design Collapsible Thinking Accordion in app-window.slint                         │
│  ├── Aggregate Thread L milestones into per-turn ChatMessage.thinking-steps            │
│  └── Add Top-Bar Status Pill displaying live API calls & RPM velocity                  │
│                                                                                        │
│  PHASE 4: TOOLS DRAWER & PRESENTATION POLISH                                           │
│  ├── Implement Tools Slide-Out Drawer with schema cards                                │
│  ├── Reverse ListView ordering to standard Bottom-Pinned with auto-scroll              │
│  └── Add Bubble Action Dock (Copy to Clipboard, Markdown Code Block formatting)        │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Phased Delivery Milestone Matrix

| Sprint Phase | Milestone Scope | Critical Files Affected | Success Verification Criteria |
| :---: | :--- | :--- | :--- |
| **Phase 1** | Stability & Deadlock Elimination | [`src/lib.rs`](../../../src/lib.rs), [`src/main.rs`](../../../src/main.rs) | Run 50 turns continuously with 0 thread futex deadlocks; double-clicks do not drop UI frame rate |
| **Phase 2** | Native Slash Commands & Errors | `src/slash_commands.rs`, [`src/slint_py_engine.rs`](../../../src/slint_py_engine.rs) | `/clear` purges Python state; `/exit` cleans and quits; API errors render red error bubbles |
| **Phase 3** | Gemini-Style Thinking Accordion | [`ui/app-window.slint`](../../../ui/app-window.slint), [`src/main.rs`](../../../src/main.rs) | Intermediate LangGraph nodes display in expandable accordion with latency timer |
| **Phase 4** | Tools Drawer & ListView Polish | `ui/components/`, [`ui/app-window.slint`](../../../ui/app-window.slint) | MCP tools toggleable in sidebar drawer; ListView pins to bottom with auto-scroll |

---

## 6. Learning Outcomes & Architectural Invariants

### Architectural Invariants Summary

| Invariant Rule | Operational Enforcement | Fatal Hazard Prevented |
| :--- | :--- | :--- |
| **1. Bounded Channels Demand Dedicated Consumers** | All `sync_channel` receivers must be drained by active loops | Thread hangs in kernel futex lock when buffer limit is exceeded |
| **2. Decouple Raw Telemetry from UI Views** | Core emits untyped raw logs; Rust parses into strongly-typed state | UI rendering drops frames when flooded with high-frequency logs |
| **3. Turn Failures Are First-Class Data** | `status == "FAILED"` maps to typed `is_error` UI structures | Blank chat bubbles that confuse users and discard diagnostics |
| **4. Thread 0 Is Sacred (Zero Blocking Calls)** | Replace `send()` with `try_send()`; never acquire GIL on UI thread | Desktop window freezing at 0 FPS with "Not Responding" alerts |

---

## 7. Quick Reference & Diagnostic Glossary

### Diagnostic Terminology & Type Contracts

| Concept / Symbol | Language / Layer | Runtime Role & Concurrency Contract |
| :--- | :--- | :--- |
| **`Bound<'py, T>`** | Rust (PyO3) | Reference bound to the active GIL lifetime `'py`. Cannot cross thread boundaries. |
| **`Py<T>`** | Rust (PyO3) | Detached reference pointer on CPython heap with `'static` lifetime. Stored in Rust structs. |
| **`slint::Weak<AppWindow>`** | Rust (Slint) | Thread-safe (`Send + Sync`) weak handle used to schedule closures via `invoke_from_event_loop`. |
| **`TicketIn`** | Python / Rust | Immutable turn input contract containing user message, timeout, and session dictionary. |
| **`TicketOut`** | Python / Rust | Turn output contract containing `status` (`COMPLETED` \| `FAILED`), `transcript`, and latency. |
| **`ReceiverLogEntry`** | Rust | Receiver end of MPSC logging bridge intercepting structured diagnostic telemetry. |

---

## 8. Q&A Log

| # | Question | Answer (concrete) | Date |
| :---: | :--- | :--- | :---: |
| **Q1** | Why not send `LogEntry` records directly from Python to Thread 0 using `slint::invoke_from_event_loop` inside `rust_handle_overrite`? | `invoke_from_event_loop` queues closures directly onto Slint's main event queue. High-frequency agent graphs emit 30–50 log events per second. Flooding the UI event loop with micro-updates forces continuous GPU repainting, dropping frame rates below 60 FPS. Buffering through a native MPSC channel to Thread L enables throttling and batching updates to the UI at a stable 10 Hz refresh rate. | 2026-10-31 |
| **Q2** | Why should `/clear` and `/exit` be parsed natively in Rust instead of passing them into Core? | Passing `/clear` or `/exit` through `TicketIn` consumes unnecessary processing cycles (graph compilation, node assignment, and classifier evaluation). Intercepting them in native Rust guarantees a 0 ms response time, cleanly drops GUI models without heap leaks, and protects external LLM rate limits. | 2026-10-31 |
| **Q3** | Which of the 7 gaps is the most dangerous and must be eliminated first? | **Gap 1 (`sync_channel(1024)` deadlock)** is the fatal priority: because the receiver is never drained, sending 1,024 log records permanently freezes the Python worker thread in an unrecoverable kernel futex lock. **Gap 4 (blocking `send` on Thread 0)** is the co-fatal priority: double-clicking Send locks the UI window immediately. | 2026-10-31 |
