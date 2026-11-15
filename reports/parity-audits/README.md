# Parity Audits Shelf

## Domain
- **desktop** (`ai-workflow-gui` Rust + Slint UI) — Parity tracking between Desktop CLI and GUI platforms.
- **shared** (`coldwind-core` PyO3 FFI boundary) — Gap analysis across concurrency, telemetry, and error recovery.

## Overview

> [!IMPORTANT]
> **Latest Codebase State:** Parity gaps `GAP-01` (bounded log deadlock) and `GAP-07` (inverted chat list) are **RESOLVED**. Parity gaps `GAP-02` (slash commands), `GAP-03` (in-flight error bubbles), `GAP-04` (blocking `state_sender.send` UI lock), `GAP-05` (`TOOL_INFO_LIST` unbound to UI), and `GAP-06` (`SlintExceptionHandler`) are strictly **OPEN** for Phase 0.0.2.

The `parity-audits/` shelf maintains forensic audit records comparing the CLI execution engine with the Slint desktop interface.

### Which File Should I Open?

| If You Are Looking For... | Open This File |
| :--- | :--- |
| Complete GAP-01 through GAP-07 audit findings, root causes, and remediation plans | [`./01-slint-gui-architecture-gap-audit.md`](./01-slint-gui-architecture-gap-audit.md) |
| Interactive prototype for the Agent Workflow Inspector drawer | [`./assets/agent-workflow-inspector-prototype.html`](./assets/agent-workflow-inspector-prototype.html) |

## Interdependence Map

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        PARITY AUDIT TOPOLOGY                           │
│                                                                        │
│   01-slint-gui-architecture-gap-audit.md                               │
│           │                                                            │
│           ├──► GAP-01 & GAP-07 ──► RESOLVED (Phase C Shipped)          │
│           │                                                            │
│           ├──► GAP-02..06      ──► OPEN (Phase 0.0.2 Roadmap)          │
│           │                                                            │
│           └──► Prototype Assets ──► assets/inspector-prototype.html    │
└────────────────────────────────────────────────────────────────────────┘
```

## Ownership Table

| Domain | Folder | Source Files Governed | Documents |
| :--- | :--- | :--- | :--- |
| `desktop` / `shared` | `./` | `../../src/main.rs`, `../../src/slint_py_engine.rs`, `../../ui/ticket-drawer.slint` | [`./01-slint-gui-architecture-gap-audit.md`](./01-slint-gui-architecture-gap-audit.md) |
| `desktop` | `./assets/` | `../../ui/ticket-drawer.slint` | [`./assets/agent-workflow-inspector-prototype.html`](./assets/agent-workflow-inspector-prototype.html) |

## Index
- [01 — Slint GUI Architecture Gap & Telemetry Parity Audit](./01-slint-gui-architecture-gap-audit.md)
- [assets/ — Interactive Workflow Inspector Prototype](./assets/agent-workflow-inspector-prototype.html)
