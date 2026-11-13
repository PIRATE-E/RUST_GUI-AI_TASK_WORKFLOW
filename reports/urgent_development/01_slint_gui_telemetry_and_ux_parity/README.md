# Urgent Development — Slint GUI Telemetry & Modern UX Parity

## Domain

- **urgent_development** (Parent Domain) — High-priority architectural audits, critical concurrency hazard fixes, and platform parity blueprints.
- **01_slint_gui_telemetry_and_ux_parity** (Sub-domain) — Forensic analysis of architectural gaps between reference Desktop CLI and native Slint GUI, logging sink deadlocks, three-thread telemetry streaming, 75% slide-in inspector drawer, and Phase C ACID disk persistence.

---

## At a glance

### Foundation Architecture & Parity Blueprint

| Document | Type | Focus / What it answers | Status |
| :--- | :--- | :--- | :---: |
| [`01_slint_gui_architecture_gap_and_telemetry_audit.md`](./01_slint_gui_architecture_gap_and_telemetry_audit.md) | Investigation & Gap Analysis | Forensically compares Desktop CLI vs Slint GUI, isolates 7 fatal gaps (deadlock, freezes, blank bubbles), and defines the 4-pillar parity blueprint. | 🟢 Active |
| [`02_threading_and_live_telemetry_pipeline.md`](./02_threading_and_live_telemetry_pipeline.md) | Design / Decision | Decouples GIL-blocked `process_ticket` from log consumption via Thread 3 (Telemetry Drainer), batch-draining logs for smooth 60 FPS in-place updates. | 🟢 Approved |
| [`03_bubble_and_drawer_state_machine.md`](./03_bubble_and_drawer_state_machine.md) | Design / Decision | **TIER 1 (RAM ONLY):** Governs in-flight live turns. Defines the 4-state matrix (`is_generating` × `drawer_open`) using volatile RAM `ActiveTurnBuffer` with zero disk I/O. | 🟢 Approved |
| [`04_disk_persistence_and_cold_reload.md`](./04_disk_persistence_and_cold_reload.md) | Design / Decision | **TIER 2 (DISK STORAGE):** Governs completed historical turns. High-level architecture for flushing finalized logs to append-only `logs/<id>.jsonl` for cold reloads. | 🟢 Approved |

### Phase C Implementation Deep-Dives (`phase_c_impl/`)

| Document | Type | Focus / What it answers | Status |
| :--- | :--- | :--- | :---: |
| [`phase_c_impl/README.md`](./phase_c_impl/README.md) | **Sub-Domain Index** | Overview of Phase C disk persistence, ACID swap-file lifecycle, and 144 FPS cold historical replay topology. | `VERIFIED` ✅ |
| [`phase_c_impl/01-serde-serialization-architecture.md`](./phase_c_impl/01-serde-serialization-architecture.md) | Concept Note & Design | Why Rust erases struct field names at compile time, why `serde` and `serde_json` are split, AST derive macros, and `#[serde(untagged)]` on `MetaData`. | `VERIFIED` ✅ |
| [`phase_c_impl/02-acid-swap-file-ingestion.md`](./phase_c_impl/02-acid-swap-file-ingestion.md) | Design & Systems | Maps Python `open()` to Rust's 3-layer I/O (`OpenOptions`, `File`, `BufWriter`). Implements `write(2)` + `fsync(2)` + atomic `renameat2(2)` commit. | `VERIFIED` ✅ |
| [`phase_c_impl/03-cold-replay-and-edge-cases.md`](./phase_c_impl/03-cold-replay-and-edge-cases.md) | Design & Investigation | How `on_inspect_ticket` streams historical turns on demand via `BufReader::lines()`, protects the 144 FPS frame budget, and recovers crashed `.swap` files. | `VERIFIED` ✅ |

---

## Master Architecture Map

```mermaid
flowchart TD
    Send["User Clicks Send (main.rs)"] --> D02["02: Threading & Live Telemetry"]
    D02 --> T2["Thread 2: Python process_ticket (GIL Blocked)"]
    D02 --> T3["Thread 3: Log Receiver & Batch Drainer"]
    T3 --> D03["03: 4-State Bubble & Drawer Machine"]
    D03 --> Closed["Drawer Closed: Update Active Bubble Row Only"]
    D03 --> Open["Drawer Open: Update Bubble Row + Push Drawer Model"]
    T3 --> PC01["Phase C: 01-serde LogEntry Serialization"]
    PC01 --> PC02["Phase C: 02-acid Ingestion (logs/<id>.swap -> BufWriter)"]
    PC02 --> Rename["Turn Done: flush() + fsync() + fs::rename -> <id>.jsonl"]
    Rename --> PC03["Phase C: 03-cold Replay (BufReader::lines)"]
    PC03 --> ColdClick["User Clicks [>] on Old Bubble -> 144 FPS Replay"]
```

---

## Key takeaways

1. **Deadlock Hazard (Gap 1):** The logging bridge initializes `sync_channel::<LogEntry>(1024)`, but Thread 2 blocks on Python. Moving log ingestion into a dedicated **Thread 3** continuously drains the channel and prevents kernel futex hangs.
2. **Deterministic `ticket_id` Handoff:** Pre-calculating `ticket_id` on Thread 0 before dispatch guarantees that the UI Sentinel bubble, the Python worker, and Thread 3 share an identical lookup key from millisecond zero.
3. **In-Place Row Mutation:** Using `chat_model_rc.set_row_data(last_idx, updated_msg)` inside `invoke_from_event_loop` eliminates list flickering and avoids pushing duplicate AI bubbles.
4. **4-State Inspection Guard:** Checking `active_ticket.ticket_id == current_ticket_id` before routing logs to the drawer allows users to safely inspect historical turns without background streaming logs corrupting the view.
5. **Phase C ACID Disk Persistence:**
   - **Domain Integrity:** Serializes native `LogEntry` directly, preserving typed metrics and dictionaries (never lossy `DrawerLogItem` strings).
   - **ACID Isolation:** Active turns write strictly to `logs/<ticket_id>.swap`; only completed turns are atomically committed to `logs/<ticket_id>.jsonl` using `flush()` + `sync_all()` (`fsync`) + `std::fs::rename`.
   - **144 FPS Protection:** Historical tickets are streamed line-by-line using `BufReader::lines()` with flat heap memory and safely tolerating truncated lines from crashed sessions.

---

## Reading path

1. Start with [`01_slint_gui_architecture_gap_and_telemetry_audit.md`](./01_slint_gui_architecture_gap_and_telemetry_audit.md) for the baseline audit and comparative gap matrix.
2. Study [`02_threading_and_live_telemetry_pipeline.md`](./02_threading_and_live_telemetry_pipeline.md) for the three-thread concurrency model.
3. Follow with [`03_bubble_and_drawer_state_machine.md`](./03_bubble_and_drawer_state_machine.md) to understand drawer state permutations and active turn buffering.
4. Review [`04_disk_persistence_and_cold_reload.md`](./04_disk_persistence_and_cold_reload.md) for the Tier 2 disk storage conceptual model.
5. **Deep Dive into Phase C Implementation (`phase_c_impl/`):**
   - Start with [`phase_c_impl/01-serde-serialization-architecture.md`](./phase_c_impl/01-serde-serialization-architecture.md) for compile-time AST macros, pointer layouts, and `#[serde(untagged)]`.
   - Read [`phase_c_impl/02-acid-swap-file-ingestion.md`](./phase_c_impl/02-acid-swap-file-ingestion.md) for Linux syscall mappings (`OpenOptions`, `File`, `BufWriter`, `fsync`, atomic rename).
   - Read [`phase_c_impl/03-cold-replay-and-edge-cases.md`](./phase_c_impl/03-cold-replay-and-edge-cases.md) for line-by-line streaming replay and `.swap` crash recovery.
6. Inspect the HTML reference prototype at [`agent_workflow_inspector_prototype.html`](./agent_workflow_inspector_prototype.html).
7. Return to parent domain catalog at [`../README.md`](../README.md).
