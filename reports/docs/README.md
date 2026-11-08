# 📚 Codebase Architecture & File Explanations (`reports/docs/`)

| Metric | Specification |
| :--- | :--- |
| **Domain** | Comprehensive Codebase Documentation Library |
| **Path** | `reports/docs/README.md` |
| **Scope** | Exhaustive file-by-file architecture breakdowns across UI, Rust business logic, FFI engine, and telemetry |
| **Status** | 🟢 Active |
| **Last Updated** | 2026-11-04 |

> **TL;DR:** `reports/docs/` contains deep, beginner-accessible, section-by-section documentation of the live codebase. Every file in the repository is broken down line-by-line, decoding every keyword and connecting components across the Slint (UI) $\leftrightarrow$ Rust (Native) $\leftrightarrow$ PyO3 (CPython 3.14) tri-layer bridge.

---

## 🗂️ Documentation Domains

```text
reports/docs/
│
├── README.md                      <- Codebase Documentation Root
│
├── ui/                            <- Declarative UI & Slint Components (6 Guides)
│   ├── README.md                  <- UI Domain Index & Android Rosetta Stone
│   ├── 01_types_and_contracts.md  <- ui/types.slint (AppState, ChatMessage, DrawerLogItem)
│   ├── 02_root_window_and_navigation.md <- ui/app-window.slint (AppWindow, layouts, overlays)
│   ├── 03_chat_bubble_orchestration.md  <- ui/chat-bubble.slint (Polymorphic router)
│   ├── 04_user_bubble_component.md      <- ui/chat_bubbles/user-bubble.slint (Prompt card)
│   ├── 05_ai_telemetry_bubble.md        <- ui/chat_bubbles/ai-bubble.slint (Sentinel console)
│   └── 06_telemetry_drawer.md           <- ui/ticket-drawer.slint (75% widescreen drawer)
│
└── src/                           <- Rust Core, FFI & Concurrency (6 Guides)
    ├── README.md                  <- Systems Domain Index & CPython Rosetta Stone
    ├── 01_lib_core_types_and_channels.md        <- src/lib.rs (Types, MetaData, Channel)
    ├── 02_main_orchestrator_and_multithreading.md <- src/main.rs (4-Thread Actor Model)
    ├── 03_cpython_boot_and_venv_injection.md   <- src/load_python.rs (Venv & Boot)
    ├── 04_pyo3_descriptor_monkeypatching.md     <- src/log_entry_handler.rs (Trampolines)
    ├── 05_slint_py_engine_runtime_facade.md    <- src/slint_py_engine.rs (SlintRuntime)
    └── 06_status_quote_engine.md               <- src/status_quote_engine.rs (Quotes)
```

### Domain Catalog

| Domain | Directory | Focus & Contents | Target Audience | Status |
| :--- | :--- | :--- | :--- | :--- |
| **UI Components** | [`ui/`](./ui/README.md) | Exhaustive breakdown of all 6 `.slint` files, layout hierarchies, two-way bindings, 60 FPS animations, and Android Studio mental models. | Android XML / Kotlin developers learning Slint | 🟢 Complete (6 Docs) |
| **Rust Core & FFI Engine** | [`src/`](./src/README.md) | In-depth breakdown of all 6 Rust source files, 4-thread actor pipeline, descriptor monkey-patching, CPython 3.14 FFI, and indigenous quotes. | Python & Systems engineers mastering PyO3 | 🟢 Complete (6 Docs) |

---

## 🎓 Pedagogical Philosophy

1. **Zero-Vibe Coding:** Every component is explained down to its exact memory layout, ownership semantics, and thread safety.
2. **The Android Bridge:** UI concepts are explained through Android Studio (XML layouts, `ViewBinding`, `RecyclerView`, `Looper`/`Handler`, `ObjectAnimator`)—never web or HTML/CSS analogies.
3. **Line-by-Line Transparency:** Every keyword, macro, and type is decoded in plain English with physical metaphors and practical failure scenarios.
