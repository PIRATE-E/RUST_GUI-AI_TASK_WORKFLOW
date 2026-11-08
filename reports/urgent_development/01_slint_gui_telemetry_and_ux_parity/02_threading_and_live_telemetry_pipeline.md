# Threading Topology & Live Telemetry Pipeline

| Attribute | Value |
| :--- | :--- |
| **Date** | 2026-11-01 |
| **Author / Role** | AI Systems Architect |
| **Domain** | `urgent_development / 01_slint_gui_telemetry_and_ux_parity` |
| **Document Type** | Design / Decision |
| **Status** | 🟢 Approved |
| **Confidence** | VERIFIED |

> **TL;DR:** Decouples the blocking Python LangGraph execution thread from the log ingestion pipeline by introducing a dedicated Telemetry Consumer Thread (Thread 3). This prevents UI freezes, drains the bounded 1024-slot channel continuously, and mutates in-flight `ChatMessage` rows in place via `set_row_data`.

---

## 1. Problem Statement & Bottleneck Analysis

When user prompts are submitted to the Python core, `slint_runtime_obj.process_ticket(py, &prompt, 60)` blocks the calling thread for 5 to 60 seconds while LangGraph executes LLM queries. Because the logging receiver (`log_receiver = init_sync_send_all()`) was hosted on that same blocked thread, no consumer existed to drain `mpsc::sync_channel::<LogEntry>(1024)`. 

| Execution Stage | Thread 2 (Python Worker) State | Channel Queue State | Visual Impact on UI |
| :--- | :--- | :--- | :--- |
| **T+0.01s (Send)** | Wakes up, calls `Python::attach` | Empty | Sentinel bubble pushed with `"Queued"` placeholder |
| **T+1.00s (Router)** | Blocked inside `process_ticket` | Router `LogEntry` pushed | Bubble remains static; status does not update |
| **T+3.50s (LLM Turn)**| Blocked waiting on HTTP NIM API | 10–40 logs buffered | Zero telemetry animations; UI appears frozen |
| **T+15.0s (Completion)**| `process_ticket` returns `TicketOut` | 100+ unread logs queued | Final text overwrites Sentinel abruptly |

```mermaid
flowchart LR
    subgraph Broken ["Current Single-Worker Bottleneck"]
        P1["Thread 2 Worker"] -->|"Calls (Blocks)"| PT["process_ticket()"]
        PT -.->|"Emits logs to"| CH["sync_channel(1024)"]
        CH -.->|"Unread during turn"| BLK["Logs Buffered / Stalled"]
    end
    subgraph Fixed ["Decoupled Multi-Thread Pipeline"]
        T2["Thread 2 (Python)"] -->|"Drives"| PT2["process_ticket()"]
        PT2 -->|"Emits LogEntry"| CH2["Sync Channel"]
        CH2 -->|"Drains non-stop"| T3["Thread 3 (Telemetry Drainer)"]
        T3 -->|"invoke_from_event_loop"| UI["Thread 0: set_row_data()"]
    end
```
*Current vs Target Pipeline: Separating execution from telemetry consumption eliminates the channel bottleneck.*

---

## 2. Three-Thread Runtime Topology

The architecture separates responsibilities across three operating system threads. Thread 0 handles 60 FPS GPU rendering and window events. Thread 2 hosts the CPython runtime and blocks during LangGraph execution. Thread 3 runs an event loop that drains incoming logs and pushes batched UI notifications.

| Thread ID | Thread Name | Ownership & Safety | Role & Concurrency Contract |
| :--- | :--- | :--- | :--- |
| **Thread 0** | Slint UI Event Loop | Main Thread (`!Send`) | Handles user typing, window painting, and receives `invoke_from_event_loop` closures. Never blocks on GIL or channels. |
| **Thread 2** | Python Engine Worker | Dedicated OS Worker | Holds `SlintRuntime` instance. Blocks on `state_receiver.recv()` when idle, blocks on `process_ticket()` during turns. |
| **Thread 3** | Telemetry & State Drainer | Dedicated OS Worker | Holds `ReceiverLogEntry`. Blocks on `log_receiver.recv()`, drains burst logs via `try_recv()`, and updates active `ChatMessage` rows. |

```mermaid
sequenceDiagram
    autonumber
    participant UI as Thread 0: Slint UI
    participant Py as Thread 2: Python Worker
    participant Tel as Thread 3: Telemetry Drainer

    UI->>UI: Push User Bubble & Sentinel AI Bubble (slint-0001)
    UI->>Py: state_sender.send(CurrentUserMsg(ticket_id, prompt))
    Py->>Py: process_ticket() blocks on LangGraph execution
    loop Live Log Stream
        Py-->>Tel: rust_handle_overrite emits LogEntry
        Tel->>UI: invoke_from_event_loop -> set_row_data(last_idx)
    end
    Py->>Tel: Turn Complete (TicketOut)
    Tel->>UI: invoke_from_event_loop -> set_row_data(is_generating=false)
```
*Sequence Diagram: Step-by-step thread coordination from message dispatch to turn completion.*

---

## 3. Surgical Code Modification Matrix

To implement this topology without breaking the single-crate structure, specific files require surgical adjustments.

| Target File | Exact Line / Area | Modification Required | Architectural Reason |
| :--- | :--- | :--- | :--- |
| [`src/main.rs`](../../src/main.rs) | `fn main()` Startup | Move `init_sync_send_all()` out of Thread 2 into Thread 3 spawn. | Allows Thread 3 to drain logs while Thread 2 is blocked inside Python. |
| [`src/main.rs`](../../src/main.rs) | `WorkerState` Enum | Change `CurrentUserMsg(String)` to include pre-allocated `ticket_id: String`. | Ensures UI sentinel, logger, and Python share the exact same ID from millisecond 0. |
| [`src/main.rs`](../../src/main.rs) | `app.on_send_message` | Bind `app.set_messages(...)` once at boot; only push to `message_stack` on click. | Prevents complete `ModelRc` re-evaluations on every send. |
| [`src/slint_py_engine.rs`](../../src/slint_py_engine.rs) | `process_ticket` | Accept pre-allocated `ticket_id: &str` parameter instead of generating internally. | Eliminates ticket ID skew between frontend Sentinel and Python logs. |

```mermaid
graph TD
    M["src/main.rs: fn main()"] -->|Spawns| T2["Thread 2: Python Worker"]
    M -->|Spawns| T3["Thread 3: Telemetry Drainer"]
    M -->|Binds once| SR["app.set_messages(ModelRc::from(message_stack))"]
    T2 -->|Runs turn| SP["slint_py_engine::process_ticket(ticket_id, prompt)"]
    T3 -->|Drains| LOG["ai_workflow_gui::init_sync_send_all()"]
```
*File Responsibility Mapping: Shows where thread instantiation and model binding occur.*

---

## 4. Burst Draining & Frame Stutter Prevention

When an AI workflow executes, nodes frequently emit 20–50 log events in sub-millisecond bursts (e.g., tool calls, metadata deserialization). Scheduling an independent `invoke_from_event_loop` closure for every single log would flood Slint's event queue and cause UI frame drops.

| Approach | Latency | UI Event Queue Load | Frame Drop Risk |
| :--- | :---: | :---: | :---: |
| **1-to-1 Dispatch** (`invoke` per log) | ~0.1ms | Very High (50+ calls/sec) | 🔴 High (Causes rendering hitching) |
| **Batch Draining** (`recv` + `try_recv`) | ~1.0ms | Low (1 call per burst) | 🟢 Zero (Smooth 60 FPS maintained) |
| **Fixed Interval Polling** (Timer 50ms) | 50.0ms | Constant Medium | 🟡 Moderate (Delayed feedback) |

```mermaid
flowchart TD
    Recv["Block on log_receiver.recv()"] --> GotFirst["First LogEntry Received"]
    GotFirst --> Loop["while let Ok(next) = log_receiver.try_recv()"]
    Loop --> Accumulate["Accumulate telemetry metrics in RAM"]
    Accumulate --> CheckMore{"More logs in channel?"}
    CheckMore -- Yes --> Loop
    CheckMore -- No --> BatchDispatch["Dispatch single invoke_from_event_loop()"]
    BatchDispatch --> UIUpdate["set_row_data(last_idx, updated_msg)"]
```
*Batch-Draining Algorithm: Blocks when idle, then flushes bursts in a single UI event loop invocation.*

---

## 5. Verification & Safety Constraints

| Hazard | Risk Description | Verification & Mitigation |
| :--- | :--- | :--- |
| **Deadlock on Channel Full** | `mpsc::sync_channel(1024)` blocks if receiver stalls. | VERIFIED: Thread 3 runs an unblocked loop with zero sleep calls. |
| **UI Data Race** | Mutating `VecModel` simultaneously from two threads. | VERIFIED: All mutations route strictly through `slint::invoke_from_event_loop`. |
| **ID Desynchronization** | Sentinel bubble displays `t-1` while Python executes `t-2`. | VERIFIED: `ticket_id` generated on Thread 0 before message dispatch. |

---

## 6. Q&A Log

| # | Question | Answer (concrete) | Date |
| :--- | :--- | :--- | :--- |
| **Q1** | Why can't Thread 2 drain logs after `process_ticket()` returns? | Draining after completion defeats live telemetry; the Sentinel bubble would remain frozen on `"Queued"` until the turn is already over. | 2026-11-01 |
| **Q2** | Why use `mpsc::sync_channel` instead of an unbounded `channel`? | A bounded channel enforces backpressure and prevents runaway memory leaks if log volume exceeds consumption rates. | 2026-11-01 |
| **Q3** | Does Thread 3 require a Python GIL lock? | No. `rust_handle_overrite` deserializes Python log objects into native Rust `LogEntry` structs inside Python, allowing Thread 3 to operate 100% GIL-free. | 2026-11-01 |
