# Foundations Shelf

## Domain
- **shared** (PyO3 0.26+ FFI boundary & CPython 3.14 internals) — Rust ↔ Python systems fundamentals.
- **desktop** (`ai-workflow-gui` reactive UI foundations) — Slint 1.16 declarative markup & Android bridges.

## Overview

> [!IMPORTANT]
> **Latest Codebase State:** The `foundations/` shelf preserves the gold-standard educational guides and Tier 1 cheat sheets for PyO3 CPython 3.14 FFI and Slint UI, anchored by The Sheet Law (`00-authoring-and-sheet-law.md`).

This shelf houses first-principles tutorials, visual memory models, and Rosetta Stone guides for engineers learning Rust, Slint, and PyO3 concurrently.

### Which File Should I Open?

| If You Are Looking For... | Open This File |
| :--- | :--- |
| Canonical Gold-Standard Authoring & Cheat Sheet Law | [`./00-authoring-and-sheet-law.md`](./00-authoring-and-sheet-law.md) |
| Complete PyO3 FFI guides, GIL lifecycles, and `PyTypeObject` descriptors | [`./cross-platform/README.md`](./cross-platform/README.md) |
| Slint WhatsApp bubble styling, component models, and reactive properties | [`./slint-active/README.md`](./slint-active/README.md) |

## Interdependence Map

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        FOUNDATIONS TOPOLOGY                            │
│                                                                        │
│   00-authoring-and-sheet-law.md (Master Authoring Law & Invariants)    │
│           │                               │                            │
│           ▼                               ▼                            │
│   cross-platform/                         slint-active/                │
│   (PyO3 0.26+, GIL, Trampolines,          (WhatsApp Bubble Styling,    │
│    Memory Models, C1–C12 Cheat Sheets)     Models, Smart Pointers)     │
└────────────────────────────────────────────────────────────────────────┘
```

## Ownership Table

| Domain | Folder | Source Files Governed | Documents |
| :--- | :--- | :--- | :--- |
| `governance` | `./` | `../../AGENTS.md` | [`./00-authoring-and-sheet-law.md`](./00-authoring-and-sheet-law.md) |
| `shared` | `./cross-platform/` | `../../src/load_python.rs`, `../../src/slint_py_engine.rs` | [`./cross-platform/`](./cross-platform/README.md) |
| `desktop` | `./slint-active/` | `../../ui/**/*.slint` | [`./slint-active/`](./slint-active/README.md) |

## Index
- [00 — Master Authoring & Cheat Sheet Law](./00-authoring-and-sheet-law.md)
- [cross-platform/ — Cross-Platform PyO3 & Slint Concurrency Bridge](./cross-platform/README.md)
- [slint-active/ — Active Slint UI Implementation & WhatsApp Bubbles](./slint-active/README.md)
