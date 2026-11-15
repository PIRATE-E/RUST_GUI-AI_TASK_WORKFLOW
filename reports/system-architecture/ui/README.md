# 🎨 UI Architecture & Slint Component Library (`ui/`)

| Metric | Specification |
| :--- | :--- |
| **Domain** | Slint UI Declarative Design & Component Architecture |
| **Path** | `reports/system-architecture/ui/README.md` |
| **Target Audience** | Developers with Android Studio background (XML, `ViewBinding`, `RecyclerView`, `Looper`/`Handler`) |
| **Files Documented** | `types.slint`, `app-window.slint`, `chat_bubbles/user-bubble.slint`, `chat_bubbles/ai-bubble.slint`, `ticket-drawer.slint` |
| **Status** | 🟢 Complete (6 Reports) |
| **Last Updated** | 2026-11-15 |

> **TL;DR:** This domain provides an exhaustive, section-by-section breakdown of every `.slint` file in `ai-workflow-gui`. It anchors declarative Slint concepts (properties, callbacks, layout trees, animations, virtualized list models) directly to familiar Android Studio mental models (XML layouts, `ViewBinding`, `RecyclerView.Adapter`, `ObjectAnimator`).

---

## 🗺️ 1. UI Component Hierarchy & Architecture Map

Every visual element in the desktop application is composed into a strict reactive tree rooted in `AppWindow`:

```text
+-----------------------------------------------------------------------------+
|                           SLINT UI COMPONENT MAP                            |
+-----------------------------------------------------------------------------+

                      ui/types.slint
            (AppState, ChatMessage, DrawerLogItem)
                            │
                            │ Imported by
                            ▼
                   ui/app-window.slint
                (Root Window Container)
                            │
         ┌──────────────────┼──────────────────────────┐
         ▼                  ▼                          ▼
   Header Bar         ListView Feed             Bottom Input Dock
  (Status Orb &      (for msg in root.messages)  (LineEdit + Send)
   App Branding)            │
                            ▼
              Chat Bubble Component Router
            (ui/chat_bubbles/ai-bubble.slint)
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
 [if msg.is-user == true]        [if msg.is-user == false]
            │                               │
            ▼                               ▼
ui/chat_bubbles/user-bubble.slint  ui/chat_bubbles/ai-bubble.slint
  (Indigo Prompt Card, 340px)       (Cyber-Slate Telemetry Card, 430px)
                                    • Live Status Dot (Amber/Emerald/Red)
                                    • Task Progress Pill [1/3]
                                    • ⚡ Dual RPM/Total API Velocity
                                    • Multi-Line Word-Wrapped Sentinel
                                    • Expanding [|> Live Console] Button
                                            │
                                            │ Clicks bubble trigger
                                            ▼
                                  ui/ticket-drawer.slint
                              (75% Slide-In Telemetry Drawer)
                              • Modal Scrim Backdrop Dimmer
                              • Untruncated Widescreen Hero Console
                              • 3-Card KPI Strip (Progress, RPM, Faults)
                              • Virtualized Execution Log Stream
```

---

## 📈 2. Recommended Reading Path

To build an intuitive mental model of how Slint and Rust collaborate, read the documents in this exact order:

```text
  1. Data Contracts     2. Root Window        3. Orchestration
+-------------------+ +-------------------+ +-------------------+
| 01_types_and_     |►| 02_root_window_   |►| 03_chat_bubble_   |
| contracts.md      | | and_navigation.md | | orchestration.md  |
+-------------------+ +-------------------+ +-------------------+
                                                      │
                                                      ▼
  6. Telemetry Drawer   5. AI Telemetry Bubble 4. User Bubble Card
+-------------------+ +-------------------+ +-------------------+
| 06_telemetry_     |◄| 05_ai_telemetry_  |◄| 04_user_bubble_   |
| drawer.md         | | bubble.md         | | component.md      |
+-------------------+ +-------------------+ +-------------------+
```

1. [**01_types_and_contracts.md**](./01_types_and_contracts.md): Learn how Slint declares shared types (`ChatMessage`, `AppState`, `DrawerLogItem`) that `build.rs` compiles into native Rust structs in `$OUT_DIR`.
2. [**02_root_window_and_navigation.md**](./02_root_window_and_navigation.md): Master the root container (`AppWindow`), layout stretch constraints (`vertical-stretch: 1` vs `0`), two-way bindings (`<=>`), and conditional lifecycle screens (Booting, Ready, Failed).
3. [**03_chat_bubble_orchestration.md**](./03_chat_bubble_orchestration.md): Discover how `ChatBubble` acts as a polymorphic router (like Android's `getItemViewType`), leveraging `inherits VerticalLayout` for automatic row height measurement.
4. [**04_user_bubble_component.md**](./04_user_bubble_component.md): Inspect user prompt cards, modern Indigo gradients (`@linear-gradient`), and why a fixed 340px width prevents short-word layout collapse.
5. [**05_ai_telemetry_bubble.md**](./05_ai_telemetry_bubble.md): Dive into the 4-tier live telemetry strip, the animated hover-expanding `|>` button, and the multi-line word-wrapped Sentinel quote console.
6. [**06_telemetry_drawer.md**](./06_telemetry_drawer.md): Explore the 75% widescreen slide-over modal, coordinate-based translation animation, touch-absorbing click interceptors, and real-time KPI streaming.

---

## 📋 3. Document Index (At a Glance)

| Document | Type | Core Question Answered | Android Mental Anchor | Status |
| :--- | :--- | :--- | :--- | :--- |
| [**01_types_and_contracts.md**](./01_types_and_contracts.md) | Concept Note | How do Slint and Rust share data without serialization? | Kotlin `data class` & Room Entity | 🟢 Complete |
| [**02_root_window_and_navigation.md**](./02_root_window_and_navigation.md) | Concept Note | How does `AppWindow` lay out widgets and switch screens? | `activity_main.xml` + `MainActivity` | 🟢 Complete |
| [**03_chat_bubble_orchestration.md**](./03_chat_bubble_orchestration.md) | Concept Note | How does Slint render different bubble types in one list? | `RecyclerView.Adapter.getItemViewType` | 🟢 Complete |
| [**04_user_bubble_component.md**](./04_user_bubble_component.md) | Concept Note | How are user prompts styled and constrained? | `item_chat_user.xml` + `CardView` | 🟢 Complete |
| [**05_ai_telemetry_bubble.md**](./05_ai_telemetry_bubble.md) | Concept Note | How do live RPM and agent quotes stream into AI bubbles? | Custom `ViewHolder` + `StateFlow` | 🟢 Complete |
| [**06_telemetry_drawer.md**](./06_telemetry_drawer.md) | Concept Note | How does the 75% slide-in drawer inspect execution logs? | `BottomSheetDialogFragment` + Scrim | 🟢 Complete |

---

## 🌉 4. The Android ↔ Slint Rosetta Stone

Keep this quick-reference table handy whenever working on `.slint` files:

| Android Studio (XML / Kotlin) | Slint Equivalent | Key Difference / Gotcha |
| :--- | :--- | :--- |
| `res/layout/*.xml` | `ui/*.slint` | Slint is compiled ahead-of-time by `build.rs` into native Rust code. |
| `findViewById(R.id.txt)` | **None!** (Reactive bindings) | You never grab widgets imperatively. You mutate root properties or list models. |
| `android:layout_weight="1"` | `vertical-stretch: 1;` | Tells the layout engine to give all remaining space to this widget. |
| `android:layout_height="wrap_content"` | `inherits VerticalLayout` | Automatically calculates the intrinsic height of child elements. |
| `android:visibility="gone"` | `if condition: Component { ... }` | Slint unmounts the element completely from the render tree. |
| `@={viewModel.input}` | `text <=> root.current-input;` | Two-way binding operator between widget and parent property. |
| `button.setOnClickListener { ... }` | `callback send-message();` | Declared in Slint, wired in Rust via `app.on_send_message(...)`. |
| `notifyItemChanged(idx)` | `model.set_row_data(idx, item)` | Mutates a single row in `VecModel` in-place with zero flicker. |
| `ValueAnimator` / XML transition | `animate property { duration: 200ms; }` | Slint interpolates properties on the GPU at up to 144 Hz automatically. |
