# 🖼️ Root Window Architecture & Navigation (`ui/app-window.slint`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`ui/app-window.slint`](../../../ui/app-window.slint) |
| **Document Path** | `reports/system-architecture/ui/02_root_window_and_navigation.md` |
| **Domain** | UI Architecture & Root Window Lifecycle |
| **Target Persona** | Junior Android Developer (`MainActivity`, `activity_main.xml`, `ViewBinding`, `LinearLayout`) |
| **Android Architectural Anchor** | `MainActivity.kt` + `activity_main.xml` + `ViewBinding` + Two-Way DataBinding |
| **Verification Mode** | Codebase Verified (`ui/app-window.slint`, `src/main.rs`) |
| **Confidence** | VERIFIED |

> **TL;DR:** `ui/app-window.slint` is the root UI container of the application. It acts as both the layout blueprint (header, virtualized `ListView`, bottom dock, slide-in drawer) and the reactive property switchboard between Slint's GPU renderer and Rust's event loop.

---

## 🏛️ 1. The High-Level Anatomy of `AppWindow`

In Android Studio, you typically define your screen in `res/layout/activity_main.xml` and control it from `MainActivity.kt` using `ViewBinding`.

In Slint, `AppWindow inherits Window` is the **direct equivalent of your root `Activity` window and layout hierarchy combined into one file**.

```text
+-----------------------------------------------------------------------------+
|                          AppWindow VISUAL LAYER TREE                        |
+-----------------------------------------------------------------------------+
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | Top Header Bar (height: 44px, vertical-stretch: 0)                    |  |
|  | [●] COLD WIND AI • AGENT HARNESS            PyO3 + Slint Live Telemetry|  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | Scrollable Chat Feed (ListView, vertical-stretch: 1)                  |  |
|  |                                                                       |  |
|  |   [User Bubble] ---------------------------------------------> [Right] |  |
|  |                                                                       |  |
|  |   [Left] <-------------------------------------------- [AI Bubble]   |  |
|  |          • Status Dot | Progress [1/2] | ⚡ 45 RPM • 3 Reqs           |  |
|  |          • Multi-Line Sentinel Console ("Planning agent steps...")    |  |
|  |          • [|> Live Console] Button                                   |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | Bottom Input Dock (height: 52px, vertical-stretch: 0)                 |  |
|  | [ LineEdit: "Ask Cold Wind AI or run a workflow..." ]  [ Send Button ] |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +  |
|  : OVERLAY 1: TicketDrawer (75% Slide-In from Right, is-open: bool)      :  |
|  : OVERLAY 2: Booting Splash Screen (if app-state == AppState.Booting)    :  |
|  : OVERLAY 3: Crash Recovery Screen (if app-state == AppState.Failed)     :  |
|  + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +  |
+-----------------------------------------------------------------------------+
```

### The Android ↔ Slint Mental Bridge

| Concept | Android Studio Equivalent | Slint (`app-window.slint`) Equivalent | Architectural Purpose |
| :--- | :--- | :--- | :--- |
| **Window Frame** | Window decor / Activity Canvas | `export component AppWindow inherits Window` | Declares root desktop window properties (`min-width`, `preferred-height`, `title`). |
| **Vertical Stack** | `<LinearLayout android:orientation="vertical">` | `VerticalLayout { ... }` | Positions children from top to bottom automatically without manual pixel calculations. |
| **Space Expansion** | `android:layout_weight="1"` | `vertical-stretch: 1` | Instructs the `ListView` to consume all available window space between header and footer. |
| **Fixed Heights** | `android:layout_height="44dp"` | `height: 44px; vertical-stretch: 0;` | Prevents header and footer from expanding when the window is resized. |
| **Visibility Toggles** | `view.visibility = View.GONE` | `if condition: Component { ... }` | Slint dynamically mounts/unmounts components based on boolean expressions. |
| **Click Listeners** | `button.setOnClickListener { ... }` | `callback send-message();` | Declares event triggers in UI that Rust registers with `.on_send_message(...)`. |

---

## 🔑 2. Property Directions: `in`, `out`, and `in-out`

In Android, any Kotlin code can read or write any View property. In Slint, **every property has an explicit data-flow boundary** to protect reactive performance and prevent data races:

```text
       RUST (Thread 0)                             SLINT UI
     +-----------------+                     +-------------------+
     |                 |  --- in property --►| Read-Only in UI   |
     |                 |                     | (e.g. boot-status)|
     |                 |◄-- out property --- | Written by UI     |
     |                 |                     |                   |
     |                 |◄- in-out property -►| Two-Way Synced    |
     |                 |                     | (e.g. app-state)  |
     +-----------------+                     +-------------------+
```

### Property Contract Table in `AppWindow`

| Property | Direction | Type | Why This Direction? |
| :--- | :--- | :--- | :--- |
| `app-state` | `in-out` | `AppState` | Rust sets it on boot completion; UI can read it or transition it. |
| `boot-status` | `in` | `string` | Read-only in UI. Only Rust worker threads push status updates during cold boot. |
| `boot-error` | `in` | `string` | Read-only in UI. Displays Python initialization tracebacks if boot fails. |
| `messages` | `in` | `[ChatMessage]` | Populated and updated exclusively by Rust's `Rc<VecModel<ChatMessage>>`. |
| `drawer-open` | `in-out` | `bool` | Opened when user clicks `|> Inspect Logs`; closed when user clicks `✕` or backdrop. |
| `active-ticket` | `in-out` | `ChatMessage` | Set by Slint when user clicks a bubble; Rust also reads it to know which logs to stream. |
| `current-input` | `in-out` | `string` | Two-way bound to `LineEdit` (`text <=> root.current-input`). Clearable from Rust on Send. |

---

## 🎛️ 3. Section-by-Section Code Walkthrough

### 3.1 The Window Configuration

```slint
export component AppWindow inherits Window {
    title: "Cold Wind AI • Workflow & Telemetry Console";
    min-width: 540px;
    min-height: 500px;
    preferred-width: 760px;
    preferred-height: 640px;
    background: #090d16; // Deep Obsidian Canvas
```

- **`min-width` / `min-height`:** Enforces window constraints so the user cannot shrink the window small enough to break layout rendering.
- **`preferred-width` / `preferred-height`:** The default opening geometry on application launch.
- **`background`:** Deep Obsidian (`#090d16`), providing a high-contrast dark theme.

---

### 3.2 The Chat Feed & Directional Gravity

```slint
ListView {
    vertical-stretch: 1;

    for msg[idx] in root.messages: HorizontalLayout {
        padding-top: 6px;
        padding-bottom: 6px;
        padding-left: 6px;
        padding-right: 6px;
        alignment: msg.is-user ? end : start;

        ChatBubble {
            msg: msg;
            inspect-ticket(id) => {
                root.active-ticket = msg;
                if idx > 0 && root.messages[idx - 1].is-user {
                    root.active-ticket-user-prompt = root.messages[idx - 1].text;
                }
                root.drawer-open = true;
                root.inspect-ticket(id);
            }
        }
    }
}
```

- **`ListView`:** Virtualized scrolling list container (equivalent to Android `RecyclerView`). It only renders visible items to maintain 60 FPS performance.
- **`alignment: msg.is-user ? end : start;`:** This is **directional gravity**. If the message is from the user, it aligns to the `end` (right-hand side). If it is from the AI, it aligns to the `start` (left-hand side).
- **Previous Prompt Peeking:** When user clicks `inspect-ticket` on an AI bubble, Slint checks `if idx > 0 && root.messages[idx - 1].is-user` to automatically grab the preceding user prompt and supply it to the drawer header!

---

### 3.3 Two-Way Input Binding & Send Callbacks

```slint
input-box := LineEdit {
    horizontal-stretch: 1;
    text <=> root.current-input;
    placeholder-text: "Ask Cold Wind AI or run a workflow...";
    accepted => {
        root.send-message();
    }
}
```

- **The `<=>` Operator:** In Android DataBinding, you write `@={viewModel.input}` for two-way synchronization. In Slint, `<=>` links the input field's internal text property directly to `root.current-input`.
- **`accepted => { root.send-message(); }`:** Fired when the user presses `Enter` inside the text box. It triggers the same callback as clicking the "Send" button.

---

### 3.4 Full-Canvas Lifecycle Overlays

```slint
if root.app-state == AppState.Booting: Rectangle {
    background: #090d16;
    width: 100%;
    height: 100%;
    // ... Branded Splash Screen ...
}

if root.app-state == AppState.Failed: Rectangle {
    background: #14090c;
    width: 100%;
    height: 100%;
    // ... Error Traceback Screen ...
}
```

- In Android, developers often switch fragments or show a `ProgressBar` overlay.
- In Slint, declaring a `Rectangle { width: 100%; height: 100%; }` with an `if` condition sits directly on top of the layout tree (Z-index stacking).
- When Rust changes `app_state` to `AppState::Ready`, Slint immediately unmounts the splash rectangle, revealing the chat interface instantly with 0ms transition stutter.

---

## ⚡ 4. How Rust Controls `AppWindow`

In `src/main.rs`, Rust initializes and interacts with `AppWindow` through strongly-typed methods:

| Rust Interaction | What It Does in Slint | Android Analog |
| :--- | :--- | :--- |
| `let app = AppWindow::new()?;` | Instantiates window and compiles layout tree. | `setContentView(R.layout.activity_main)` |
| `app.set_app_state(AppState::Ready);` | Sets `app-state`, dismissing splash overlay. | `splashView.visibility = View.GONE` |
| `app.set_current_input("".into());` | Clears the `LineEdit` text box after sending. | `binding.inputBox.text.clear()` |
| `app.on_send_message(move \|\| ...)` | Registers Rust callback for Enter / Send click. | `binding.sendBtn.setOnClickListener { ... }` |
| `app.as_weak()` | Creates thread-safe weak pointer to window. | `WeakReference<MainActivity>` |

---

## ⚠️ 5. Common Traps & Pitfalls for Android Developers

### Trap 1: Forgetting that Kebab-case Becomes Snake_case
- **The Mistake:** Looking for `app.on_send-message(...)` in Rust.
- **The Reality:** Slint identifiers with dashes (e.g. `send-message`, `app-state`) are auto-translated into Rust snake_case: `app.on_send_message(...)`, `app.set_app_state(...)`.

### Trap 2: Trying to call `findViewById` on child widgets
- **The Mistake:** Trying to grab the `send-btn` directly in Rust to disable it.
- **The Reality:** Slint does **not** allow grabbing child elements from code. You never modify views directly. Instead, you declare a property (e.g. `in property <bool> send-enabled: true;`) on the root window, bind the child widget to that property, and mutate the root property in Rust.

---

## ❓ 6. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Why does `AppWindow` define `messages` as `in property <[ChatMessage]>` if Rust needs to update it? | In Slint, `in property` means "settable by the hosting language (Rust), read-only to Slint declarative expressions". Rust owns and mutates the backing `VecModel`. | 2026-11-04 |
| Q2 | How does `TicketDrawer` slide over the chat feed without breaking layout constraints? | Because `TicketDrawer` is placed at the root level of `AppWindow` after the `VerticalLayout`. In Slint, elements declared later in the file are rendered at a higher Z-index, naturally floating above earlier elements. | 2026-11-04 |
| Q3 | What thread does `on_send_message` execute on? | Thread 0 (the main UI thread). To prevent freezing the UI, long-running Python operations must be dispatched across `sync_channel` to a background worker thread. | 2026-11-04 |
| Q4 | What is the window geometry configured on AppWindow? | The window is rendered at `1280px x 720px` (16:9) dynamic best-fit scaling, supporting high-refresh rendering up to 144 Hz. | 2026-11-15 |
