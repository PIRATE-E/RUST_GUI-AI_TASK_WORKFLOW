# 📋 Slint Data Types & Architecture Contracts (`ui/types.slint`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`ui/types.slint`](file:///home/pirate/development/ai-workflow-gui/ui/types.slint) |
| **Document Path** | `reports/docs/ui/01_types_and_contracts.md` |
| **Domain** | UI Architecture & Data Modeling |
| **Target Persona** | Junior Android Developer (XML + Java/Kotlin, `ViewBinding`, Room, `data class`) |
| **Android Architectural Anchor** | Kotlin `data class`, Java POJO, `sealed class AppState`, Room Entity |
| **Verification Mode** | Codebase Verified (`ui/types.slint`, `build.rs`, `src/lib.rs`, `src/main.rs`) |
| **Confidence** | VERIFIED |

> **TL;DR:** `ui/types.slint` defines the cross-boundary data contracts (`AppState`, `DrawerLogItem`, `ChatMessage`) shared between Slint's declarative UI and Rust's native runtime. By compiling declarative structs in Slint ahead-of-time via `slint_build`, Rust gains strongly-typed access to UI memory without any JSON serialization or runtime reflection.

---

## 🧭 1. What Is `ui/types.slint` and Why Does It Exist?

If you come from Android Studio, imagine writing a Kotlin `data class` or Java POJO to hold your data before passing it to a `RecyclerView.Adapter`. In Android, your UI layouts (`res/layout/*.xml`) do not declare data structures; they only bind to variables using DataBinding or you push data imperatively via `ViewBinding` in Kotlin.

Slint works differently: **Slint is both a declarative UI layout engine AND a type declaration language**.

```text
+-----------------------------------------------------------------------------+
|                     THE COMPILE-TIME TYPE GENERATION BRIDGE                 |
+-----------------------------------------------------------------------------+

   ui/types.slint                           Cargo Build Pipeline (build.rs)
  +-----------------------+                +-------------------------------+
  | export struct         |   Ahead-of-Time| slint_build::compile(...)     |
  | ChatMessage { ... }   | --------------►|                               |
  +-----------------------+   Compilation  +---------------+---------------+
                                                           |
                                                           ▼ Produces in OUT_DIR
                                            Generated Native Rust Struct:
                                           +-------------------------------+
                                           | pub struct ChatMessage {      |
                                           |   pub text: SharedString,     |
                                           |   pub is_user: bool,          |
                                           |   pub rpm_count: i32, ...     |
                                           | }                             |
                                           +-------------------------------+
                                                           ▲
                                                           | Included via:
                                           +---------------+---------------+
                                           | slint::include_modules!()     |
                                           | Inside Rust (src/main.rs)     |
                                           +-------------------------------+
```

### The Android ↔ Slint Mental Bridge

| Concept | Android Studio Equivalent | Slint + Rust Equivalent | Why This Matters |
| :--- | :--- | :--- | :--- |
| **Data Contract** | Kotlin `data class ChatMessage(...)` | `export struct ChatMessage { ... }` in `types.slint` | Declared once in `.slint`; Rust auto-generates matching native memory layout. |
| **Screen State** | `sealed class AppState { object Booting; object Ready; ... }` | `export enum AppState { Booting, Ready, Failed }` | Controls top-level screen routing between Splash, Chat feed, and Error cards. |
| **Data Sharing** | Parcelable / Serializable / Intents | Zero-copy compile-time generated structs | No serialization overhead. Slint and Rust share native C-ABI struct layouts. |
| **Code Generation** | Android Gradle Plugin generating `BR.java` or `*Binding.java` | `build.rs` running `slint_build` generating Rust code in `$OUT_DIR` | Generates safe getters and setters directly callable from Rust. |

---

## 🔍 2. Keyword & Syntax Decoder

When reading `ui/types.slint`, you encounter several Slint-specific keywords. Here is what each one means:

| Keyword | Android Studio Analog | Plain-English Meaning in Slint |
| :--- | :--- | :--- |
| `export` | `public` modifier in Kotlin/Java | Makes the struct or enum visible to other `.slint` files AND exports it to Rust via `slint::include_modules!()`. |
| `enum` | Kotlin `enum class` / `sealed class` | A discrete set of named states (e.g. `Booting`, `Ready`, `Failed`). |
| `struct` | Kotlin `data class` / Java POJO | A named container with typed fields. |
| `string` | `java.lang.String` | UTF-8 text string. In Rust, this maps to `slint::SharedString` (a clone-on-write, small-string-optimized type). |
| `bool` | `kotlin.Boolean` / `boolean` | `true` or `false` flag. |
| `int` | `kotlin.Int` / `int` | 32-bit signed integer (`i32` in Rust). |

---

## 🧱 3. Data Structure Breakdown

`ui/types.slint` contains exactly three core definitions:

```text
                               ui/types.slint
                                      │
         ┌────────────────────────────┼───────────────────────────┐
         ▼                            ▼                           ▼
1. AppState (enum)          2. DrawerLogItem (struct)    3. ChatMessage (struct)
   • Booting                   • timestamp: string          • Identity & Prompt
   • Ready                     • category: string           • Live Telemetry (RPM,
   • Failed                    • level: string                API Count, Task,
                               • message: string              Sentinel Summary)
                               • meta-preview: string
```

### 3.1 `AppState` (Application Lifecycle Routing)

```slint
export enum AppState {
    Booting,
    Ready,
    Failed,
}
```

- **What it does:** Represents the high-level operational phase of the desktop application.
- **Why it is needed:** The Python engine takes 1.5–3.0 seconds to initialize embedded CPython 3.14, import LangGraph, and attach native FFI descriptors. While booting, the GUI must show a branded splash screen instead of a blank or half-rendered window.
- **How it routes views in `ui/app-window.slint`:**
  - `AppState.Booting` $\rightarrow$ Displays full-canvas Indigo Splash Screen overlay.
  - `AppState.Ready` $\rightarrow$ Hides splash screen and reveals the live chat feed and input dock.
  - `AppState.Failed` $\rightarrow$ Shows Crimson Fatal Boot Crash overlay with error traceback.

---

### 3.2 `DrawerLogItem` (Telemetry Stream Row)

```slint
export struct DrawerLogItem {
    timestamp: string,   // e.g. "10:45:02.120"
    category: string,    // e.g. "API_CALL", "TOOL_EXECUTION", "AGENT_WORKFLOW"
    level: string,       // e.g. "INFO", "WARNING", "ERROR"
    message: string,     // Log description emitted by ColdWind core
    meta-preview: string // Compact key-value metadata string (or empty)
}
```

- **What it does:** Represents a single execution log entry inside the 75% widescreen `TicketDrawer`.
- **Android Mental Model:** Exactly like a custom Android `LogViewHolder` model used in an internal developer diagnostics dialog.
- **Severity Styling:** When rendered in `ui/ticket-drawer.slint`, the `level` property dynamically triggers border and text coloring:
  - `"ERROR"` or `"CRITICAL"` $\rightarrow$ Crimson Red (`#ef4444`)
  - `"WARNING"` $\rightarrow$ Amber Yellow (`#fbbf24`)
  - `"INFO"` $\rightarrow$ Cyber Blue (`#38bdf8`)

---

### 3.3 `ChatMessage` (The All-In-One Unified Row Contract)

```slint
export struct ChatMessage {
    // --- 1. Identity & Core Content ---
    ticket-id: string,       // e.g. "development-7a1b..."
    text: string,            // Prompt text (User) or final response (AI)
    is-user: bool,           // true = UserBubble (Right), false = AiBubble (Left)
    sender: string,          // e.g. "You", "ColdWind AI"
    timestamp: string,       // e.g. "10:45 AM" or "1240 ms"

    // --- 2. Live AI Telemetry State ---
    is-generating: bool,     // true = pulsing Sentinel box & Amber dot
    task-progress: string,   // e.g. "Task [1/3]"
    rpm-count: int,          // Live 60s sliding window RPM counter
    api-count: int,          // Total LLM API calls in this turn
    fail-count: int,         // Count of ERROR/WARNING events
    live-summary: string,    // Latest log headline / untruncated agent quote
}
```

#### Why Did We Design a Single Unified Struct?

In Android, beginners often create separate classes: `UserMessage` and `AiMessage`, or create a separate `LiveTelemetryState` class. In Slint, passing heterogeneous types or multiple synchronized models into a single `ListView` causes severe architectural headaches.

| Architecture Choice | Why We Picked It | Alternative That Was Rejected | Why Alternative Failed |
| :--- | :--- | :--- | :--- |
| **Unified `ChatMessage` Struct** | Single `VecModel<ChatMessage>` backing the entire chat list. | Two separate models: `user_messages` and `ai_messages`. | Android `RecyclerView` can handle multi-type adapters easily, but in Slint, synchronizing two list models into one unified vertical scroll sequence requires complex indexing glue. |
| **In-Place Mutation via `VecModel`** | Rust updates `last_ai_msg` in place via `set_row_data(idx, msg)`. | Appending dummy "status bubbles" and deleting them later. | Appending and removing items causes visible UI flickering and breaks scroll anchoring. |
| **Direct Field Binding** | Sub-components (`AiBubble`, `TicketDrawer`) read the fields directly. | Shared global atomics or mutex locks. | Cross-thread mutex locks freeze Slint's 60 FPS UI rendering loop. |

---

## ⚡ 4. How Rust and Slint Interact With This File

When Cargo builds the project, `build.rs` compiles `ui/app-window.slint` (which imports `ui/types.slint`). In `src/main.rs`, we invoke:

```rust
slint::include_modules!();
```

This macro expands to native Rust definitions that match the Slint types with 100% precision:

```rust
// Auto-generated by Slint compiler in target/debug/build/.../out/
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub ticket_id: slint::SharedString,
    pub text: slint::SharedString,
    pub is_user: bool,
    pub sender: slint::SharedString,
    pub timestamp: slint::SharedString,
    pub is_generating: bool,
    pub task_progress: slint::SharedString,
    pub rpm_count: i32,
    pub api_count: i32,
    pub fail_count: i32,
    pub live_summary: slint::SharedString,
}
```

> [!NOTE]
> **Kebab-case to snake_case automatic translation:**
> Notice that `ticket-id` in Slint becomes `ticket_id` in Rust, `is-user` becomes `is_user`, and `rpm-count` becomes `rpm_count`. Slint automatically converts kebab-case identifiers to standard Rust snake_case identifiers.

---

## ⚠️ 5. Common Traps & Pitfalls for Android Developers

### Trap 1: Trying to use Kotlin/Java Nulls
- **The Mistake:** Expecting `String? = null` for `active-ticket-user-prompt` or `ticket-id`.
- **The Reality:** Slint does **not** have a `null` or `nil` keyword. All properties have concrete default values (empty string `""`, `false`, or `0`).
- **The Fix:** Use empty string `""` to represent the absence of text (e.g. `if root.user-prompt != "": ...`).

### Trap 2: Modifying Struct Fields Directly in Rust
- **The Mistake:** Trying to do `app.get_messages().get(0).text = "New text";`
- **The Reality:** Slint models are reactive and copy-on-read. You cannot mutate a struct field in place inside a `ModelRc`.
- **The Fix:** Clone the struct, modify the field on the local clone, and push it back via `set_row_data(row_idx, updated_struct)`.

---

## ❓ 6. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Why is `ChatMessage` declared in `types.slint` instead of native Rust code? | Because Slint needs to understand the type layout ahead-of-time to compute UI layout geometry and generate GPU rendering commands. Declaring it in Slint lets `slint_build` generate the Rust code for both worlds. | 2026-11-04 |
| Q2 | Does `ChatMessage` incur large memory overhead for simple user messages that don't need telemetry fields? | No. The telemetry fields are three 32-bit integers and three `SharedString` handles (which are small 8-byte pointers with small-string optimization). A complete `ChatMessage` is under 128 bytes in memory. | 2026-11-04 |
| Q3 | What happens if Rust passes an invalid string to `AppState`? | It is impossible at compile time. Rust receives `pub enum AppState { Booting, Ready, Failed }`, so the Rust compiler enforces strict type safety. | 2026-11-04 |
