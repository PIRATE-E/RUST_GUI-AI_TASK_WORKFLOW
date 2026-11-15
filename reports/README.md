# Cold Wind AI GUI — Architecture & Engineering Reports

> **ai-workflow-gui Knowledge Base**: Long-form engineering documentation, architectural audits, and first-principles learning guides for **`ai-workflow-gui`** (Rust 2024 + Slint 1.16 + PyO3 0.26).
> All shelves follow the **System-First Library Shelving Contract** (`reports/<system>/<nn-slug>.md`).

---

## 🏛️ System Shelves

| System Shelf | Domains | Purpose | Status | Index |
| :--- | :--- | :--- | :--- | :--- |
| **System Architecture** | `desktop`, `shared` | Ground-truth production architecture: 3-thread actor runtime (Threads 0, 2, 3), ACID `.swap` persistence, 144 Hz ZSTD splash engine, PyO3 FFI, and Slint UI. | `VERIFIED` | [`system-architecture/README.md`](./system-architecture/README.md) |
| **Parity Audits** | `desktop`, `shared` | Forensic gap tracking (`GAP-01`–`GAP-07`) comparing Desktop CLI vs. Slint GUI, separating resolved sprints from open Phase 0.0.2 targets. | `ACTIVE` | [`parity-audits/README.md`](./parity-audits/README.md) |
| **Foundations** | `desktop`, `shared` | First-principles tutorials & Rosetta Stones: CPython 3.14 FFI/descriptors (`src/`) and Slint declarative UI/Android bridges (`ui/`). | `VERIFIED` | [`foundations/README.md`](./foundations/README.md) |
| **Test Harness** | `desktop`, `shared` | 6-target Cargo integration suite, headless PyO3 turn verification, and 144 Hz (`6.94 ms`) rasterizer benchmarks. | `VERIFIED` | [`test-harness/README.md`](./test-harness/README.md) |

---

## 🧭 Navigation & Reading Pathways

| If You Are Looking For... | Open Shelf | Key Document |
| :--- | :--- | :--- |
| **Multithreading & Actor Concurrency** | `system-architecture/` | [`01-three-thread-actor-pipeline.md`](./system-architecture/01-three-thread-actor-pipeline.md) |
| **144 Hz ASCII Splash Screen Engine** | `system-architecture/` | [`07-ascii-splash-screen-engine.md`](./system-architecture/07-ascii-splash-screen-engine.md) |
| **ACID Telemetry & File Ingestion** | `system-architecture/` | [`05-acid-swap-file-ingestion.md`](./system-architecture/05-acid-swap-file-ingestion.md) |
| **Parity Gaps & Sprint Tracking** | `parity-audits/` | [`01-slint-gui-architecture-gap-audit.md`](./parity-audits/01-slint-gui-architecture-gap-audit.md) |
| **PyO3 FFI & GIL Mechanics Tutorials** | `foundations/` | [`cross-platform/README.md`](./foundations/cross-platform/README.md) |
| **Slint Declarative UI Tutorials** | `foundations/` | [`slint-active/README.md`](./foundations/slint-active/README.md) |
| **Integration Test Suite & Benchmarks** | `test-harness/` | [`02-splash-144hz-benchmarks.md`](./test-harness/02-splash-144hz-benchmarks.md) |

---

## 📜 Architectural Rules & Governance

- **Zero Content Loss & Monotonic Expansion:** Every production report retains 100% of its original content (`new_bytes >= old_bytes`, `new_lines >= old_lines`).
- **Tutorial Immutability:** All foundational cheat sheets and guides in `reports/foundations/` are SHA-256 byte-for-byte immutable.
- **Strict Concurrency Model:** Only 3 OS threads exist in the execution topology (`Thread 0`: Slint UI + 144 Hz timer, `Thread 2`: CPython 3.14 worker, `Thread 3`: Telemetry drainer).
- **Domain Separation:** Android Studio mental bridges are confined strictly to UI (`.slint`) docs; CPython 3.14 internals apply to Rust/FFI (`src/`) docs.
- **Repository Governance:** [`../AGENTS.md`](../AGENTS.md)
- **Master Authoring & Sheet Law:** [`foundations/00-authoring-and-sheet-law.md`](foundations/00-authoring-and-sheet-law.md)
- **Root Project Overview:** [`../README.md`](../README.md)
