# Urgent Development — High-Priority Audits & System Modernization

## Overview

The **urgent_development** domain documents high-priority architectural investigations, fatal concurrency hazard remediations, and mission-critical platform modernizations for `ai-workflow-gui`.

Documents stored here identify immediate production risks—such as cross-thread deadlocks, UI event loop lockups, and unhandled exception paths—and establish phased engineering blueprints to solve them before Phase 0.0.2 feature work.

---

## Sub-domains

| Sub-domain | Path | Focus & Deliverables | Status | Documents |
| :--- | :--- | :--- | :---: | :---: |
| **01 Slint GUI Telemetry & UX Parity** | [`01_slint_gui_telemetry_and_ux_parity/`](01_slint_gui_telemetry_and_ux_parity/README.md) | Forensic audit of Desktop CLI vs Slint GUI, 1024-slot log deadlock remediation, 3-thread telemetry pipeline, 4-state inspection matrix, and 75% drawer. | 🟢 Active | 4 |

---

## Architecture Context

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        URGENT DEVELOPMENT DOMAIN                       │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│   01_slint_gui_telemetry_and_ux_parity/                                │
│   ├── GAP-01: Bounded Log Channel Deadlock (sync_channel 1024)         │
│   ├── GAP-02: Stubbed Slash Commands (/clear, /exit)                   │
│   ├── GAP-03: Silent Failure Masking (Empty Bubble on FAILED status)   │
│   ├── GAP-04: UI Thread Lock (Blocking send() on Thread 0)             │
│   ├── GAP-05: Invisible Tool Registry (TOOL_INFO_LIST disconnected)    │
│   ├── GAP-06: Stubbed Exception Handler (Empty SlintExceptionHandler)  │
│   └── GAP-07: Inverted List Direction (ListView downward vs Top label) │
│                                                                        │
│   REMEDIATION PILLARS                                                  │
│   ├── Pillar 1: Dedicated Thread L Telemetry Demultiplexer             │
│   ├── Pillar 2: Native Rust ClientCommand Parsing Engine               │
│   ├── Pillar 3: Non-Blocking try_send Dispatch & is-processing Lock    │
│   └── Pillar 4: Registered Tools Slide-Out Drawer & UI Polish          │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Navigation & Standards

- All documents in this domain adhere strictly to the [Report Protocol](../../.agents/skills/report-protocol/SKILL.md).
- Relative links are used throughout (`./`, `../`).
- Return to the root documentation catalog at [`../README.md`](../README.md).
