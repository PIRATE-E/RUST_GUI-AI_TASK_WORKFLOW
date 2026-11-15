# System Architecture Shelf

## Domain
- **desktop** (`ai-workflow-gui` Rust + Slint UI) — Shipped production desktop application architecture.
- **shared** (`coldwind-core` PyO3 FFI boundary) — Turnkey engine interface, logging sinks, and telemetry.

## Overview

> [!IMPORTANT]
> **Latest Codebase State:** The application operates a verified **3-thread OS actor topology** (`Thread 0` UI/144Hz, `Thread 2` Python Worker, `Thread 3` Telemetry/ACID Drainer), a native **144 Hz Kyoto Dusk ASCII splash engine** with in-memory Zstandard decompression, and ACID `.swap` $\rightarrow$ `.jsonl` disk persistence. Only Threads 0, 2, and 3 exist in the execution model.

The `system-architecture/` shelf documents the ground-truth production architecture running at current `HEAD` in `ai-workflow-gui`. It bridges systems concurrency, disk persistence, and reactive Slint UI components.

### Which File Should I Open?

| If You Are Looking For... | Open This File |
| :--- | :--- |
| Concurrency topology, OS thread lifecycles, and MPSC channels | [`./01-three-thread-actor-pipeline.md`](./01-three-thread-actor-pipeline.md) |
| UI state transitions, `AppState`, AI chat bubbles, and 75% telemetry drawer | [`./02-bubble-and-drawer-state-machine.md`](./02-bubble-and-drawer-state-machine.md) |
| High-level disk persistence architecture and cold reload | [`./03-disk-persistence-overview.md`](./03-disk-persistence-overview.md) |
| Serde JSONL serialization and typing contracts | [`./04-serde-serialization-architecture.md`](./04-serde-serialization-architecture.md) |
| ACID `.swap` file streaming, 8KB buffer, and atomic `renameat2` | [`./05-acid-swap-file-ingestion.md`](./05-acid-swap-file-ingestion.md) |
| Crash-resilient cold log replay and historical recovery | [`./06-cold-replay-and-edge-cases.md`](./06-cold-replay-and-edge-cases.md) |
| 144 Hz ASCII splash screen engine, Fontdue caching, and ZSTD embedding | [`./07-ascii-splash-screen-engine.md`](./07-ascii-splash-screen-engine.md) |
| Deep-dive guides for pure Rust core source files | [`./src/README.md`](./src/README.md) |
| Deep-dive guides for Slint UI components and layout | [`./ui/README.md`](./ui/README.md) |

## Interdependence Map

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        SYSTEM ARCHITECTURE TOPOLOGY                    │
│                                                                        │
│   01-three-thread-actor-pipeline.md (Runtime Host & OS Concurrency)    │
│           │                               │                            │
│           ├──► 07-ascii-splash-screen     ├──► src/ (Core Rust FFI)    │
│           │    (144Hz ZSTD Engine)        │    (Embedded CPython 3.14) │
│           │                               │                            │
│           ▼                               ▼                            │
│   02-bubble-and-drawer-state-machine.md   03..06 (ACID Persistence)    │
│   (16:9 Window, Bubbles, Drawer)          (BufWriter, fsync, .jsonl)   │
└────────────────────────────────────────────────────────────────────────┘
```

## Ownership Table

| Domain | Folder | Source Files Governed | Documents |
| :--- | :--- | :--- | :--- |
| `desktop` | `./` | `../../src/main.rs`, `../../src/lib.rs` | [`./01-three-thread-actor-pipeline.md`](./01-three-thread-actor-pipeline.md) |
| `desktop` | `./` | `../../ui/app-window.slint`, `../../ui/ticket-drawer.slint` | [`./02-bubble-and-drawer-state-machine.md`](./02-bubble-and-drawer-state-machine.md) |
| `desktop` | `./` | `../../src/serialize_logs_in_file.rs` | [`./03-disk-persistence-overview.md`](./03-disk-persistence-overview.md) |
| `desktop` | `./` | `../../src/serialize_logs_in_file.rs` | [`./04-serde-serialization-architecture.md`](./04-serde-serialization-architecture.md) |
| `desktop` | `./` | `../../src/serialize_logs_in_file.rs` | [`./05-acid-swap-file-ingestion.md`](./05-acid-swap-file-ingestion.md) |
| `desktop` | `./` | `../../src/serialize_logs_in_file.rs` | [`./06-cold-replay-and-edge-cases.md`](./06-cold-replay-and-edge-cases.md) |
| `desktop` | `./` | `../../src/splash_screen.rs` | [`./07-ascii-splash-screen-engine.md`](./07-ascii-splash-screen-engine.md) |
| `shared` | `./src/` | `../../src/lib.rs`, `../../src/load_python.rs`, `../../src/slint_py_engine.rs` | [`./src/`](./src/README.md) |
| `desktop` | `./ui/` | `../../ui/**/*.slint` | [`./ui/`](./ui/README.md) |

## Index
- [01 — Three-Thread Actor Runtime & Concurrency Topology](./01-three-thread-actor-pipeline.md)
- [02 — Bubble and Drawer State Machine & Live Telemetry](./02-bubble-and-drawer-state-machine.md)
- [03 — Disk Persistence Architecture & Cold Reload Overview](./03-disk-persistence-overview.md)
- [04 — Serde JSONL Serialization Architecture](./04-serde-serialization-architecture.md)
- [05 — ACID Swap File Ingestion & Atomic Rotation](./05-acid-swap-file-ingestion.md)
- [06 — Cold Log Replay Engine & Crash Recovery](./06-cold-replay-and-edge-cases.md)
- [07 — High-Performance 144 Hz Kyoto Dusk ASCII Splash Screen Engine](./07-ascii-splash-screen-engine.md)
- [src/ — Rust Core Modules Deep Dive](./src/README.md)
- [ui/ — Slint UI Component Architecture](./ui/README.md)
