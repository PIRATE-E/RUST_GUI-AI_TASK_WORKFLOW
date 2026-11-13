# Urgent Development — High-Priority Audits & System Modernization

## Overview

The **urgent_development** domain documents high-priority architectural investigations, fatal concurrency hazard remediations, and mission-critical platform modernizations for `ai-workflow-gui`.

Documents stored here identify immediate production risks—such as cross-thread deadlocks, UI event loop lockups, and unhandled exception paths—and establish phased engineering blueprints to solve them before Phase 0.0.2 feature work.

---

## Sub-domains & Deliverables

| Sub-domain | Path | Focus & Deliverables | Status | Documents |
| :--- | :--- | :--- | :---: | :---: |
| **01 Slint GUI Telemetry & UX Parity** | [`01_slint_gui_telemetry_and_ux_parity/`](01_slint_gui_telemetry_and_ux_parity/README.md) | Forensic audit of Desktop CLI vs Slint GUI, 1024-slot log deadlock remediation, 3-thread telemetry pipeline, 4-state inspection matrix, 75% drawer, and Phase C ACID disk persistence. | 🟢 Active | 7 Reports + 1 Sub-domain + 1 Prototype |

---

## Curated Document Catalog

### Foundation Architecture & Parity Blueprint

| Document | Type | Core Technical Focus | Status |
| :--- | :--- | :--- | :---: |
| [`01_slint_gui_telemetry_and_ux_parity/01_slint_gui_architecture_gap_and_telemetry_audit.md`](01_slint_gui_telemetry_and_ux_parity/01_slint_gui_architecture_gap_and_telemetry_audit.md) | Investigation & Gap Analysis | Forensic comparison of Desktop CLI vs Slint GUI; isolates 7 fatal gaps (deadlocks, freezes, blank bubbles). | 🟢 Active |
| [`01_slint_gui_telemetry_and_ux_parity/02_threading_and_live_telemetry_pipeline.md`](01_slint_gui_telemetry_and_ux_parity/02_threading_and_live_telemetry_pipeline.md) | Design / Decision | Decouples GIL-blocked `process_ticket` from log consumption via dedicated Thread 3 (Telemetry Drainer). | 🟢 Approved |
| [`01_slint_gui_telemetry_and_ux_parity/03_bubble_and_drawer_state_machine.md`](01_slint_gui_telemetry_and_ux_parity/03_bubble_and_drawer_state_machine.md) | Design / Decision | **TIER 1 (RAM ONLY):** Governs in-flight live turns with 4-state matrix (`is_generating` × `drawer_open`) in RAM. | 🟢 Approved |
| [`01_slint_gui_telemetry_and_ux_parity/04_disk_persistence_and_cold_reload.md`](01_slint_gui_telemetry_and_ux_parity/04_disk_persistence_and_cold_reload.md) | Design / Decision | **TIER 2 (DISK STORAGE):** High-level design for flushing finalized logs to append-only JSONL files for cold inspection. | 🟢 Approved |

### Phase C Implementation Deep-Dives (`phase_c_impl/`)

| Document | Type | Core Technical Focus | Status |
| :--- | :--- | :--- | :---: |
| [`01_slint_gui_telemetry_and_ux_parity/phase_c_impl/README.md`](01_slint_gui_telemetry_and_ux_parity/phase_c_impl/README.md) | **Sub-Domain Index** | Architecture map and reading path for Phase C persistence, ACID swap lifecycles, and cold historical replay. | `VERIFIED` ✅ |
| [`01_slint_gui_telemetry_and_ux_parity/phase_c_impl/01-serde-serialization-architecture.md`](01_slint_gui_telemetry_and_ux_parity/phase_c_impl/01-serde-serialization-architecture.md) | Concept Note & Design | Compile-time reflection, memory layout (stack headers vs heap pointers), and `#[serde(untagged)]` on `MetaData`. | `VERIFIED` ✅ |
| [`01_slint_gui_telemetry_and_ux_parity/phase_c_impl/02-acid-swap-file-ingestion.md`](01_slint_gui_telemetry_and_ux_parity/phase_c_impl/02-acid-swap-file-ingestion.md) | Design & Systems | Maps Python `open()` to `OpenOptions` + `File` (`fd`) + `BufWriter`. Enforces `write(2)` + `fsync(2)` + atomic `renameat2(2)`. | `VERIFIED` ✅ |
| [`01_slint_gui_telemetry_and_ux_parity/phase_c_impl/03-cold-replay-and-edge-cases.md`](01_slint_gui_telemetry_and_ux_parity/phase_c_impl/03-cold-replay-and-edge-cases.md) | Design & Investigation | Streams historical JSONL via `BufReader::lines()` at 144 FPS and handles crashed `.swap` file fallback/recovery. | `VERIFIED` ✅ |

### Visual Artifacts & Prototypes

| Artifact | Type | Description |
| :--- | :--- | :--- |
| [`01_slint_gui_telemetry_and_ux_parity/agent_workflow_inspector_prototype.html`](01_slint_gui_telemetry_and_ux_parity/agent_workflow_inspector_prototype.html) | Interactive HTML Prototype | Standalone visual inspector prototype validating drawer transitions, telemetry badges, and live log layout. |

---

## Architecture Context

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                               URGENT DEVELOPMENT DOMAIN                                │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│   01_slint_gui_telemetry_and_ux_parity/                                                │
│   ├── PHASE A: Concurrency Deadlock Fix & Multi-Thread Pipeline                        │
│   │   ├── Thread 3 dedicated ingestion drains OnceLock channel continuously            │
│   │   └── In-place chat bubble row mutation eliminates UI redraw flickering            │
│   │                                                                                    │
│   ├── PHASE B: 144 FPS Telemetry Drawer & 4-State Inspection                           │
│   │   ├── Fixed-height rows (76px) + overflow: elide enable O(1) virtualization        │
│   │   └── Zero GPU texture cache thrashing (removed cache-rendering-hint: true)        │
│   │                                                                                    │
│   └── PHASE C: ACID Disk Persistence & Cold Replay (phase_c_impl/)                     │
│       ├── 01-serde: Compile-time AST derivation for LogEntry / untagged MetaData        │
│       ├── 02-acid: BufWriter streams to logs/<id>.swap; fsync + rename to .jsonl       │
│       └── 03-cold: BufReader::lines() on-demand replay protects 144 FPS budget        │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Navigation & Standards

- All documents in this domain adhere strictly to the [Report Protocol](../../.agents/skills/report-protocol/SKILL.md).
- Relative links are used throughout (`./`, `../`).
- Return to the root documentation catalog at [`../README.md`](../README.md).
