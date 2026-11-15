# 🧩 Slint Properties & Data Models Cheat Sheet & 30-Second Invariants

> **Primary Objective:** Within 30 seconds of modifying any `.slint` file or Rust host code, select the correct property direction (`in` vs `in-out`), map it to its exact Rust type (`SharedString`, `ModelRc<T>`), and wire the corresponding getter/setter/callback without compiler borrow or direction errors.
> **Master Guide:** [Active UI Reports README](README.md) · **Concept Notes:** [01_chat_stack_ui_architecture.md](01_chat_stack_ui_architecture.md)

---

## 📺 1. Curated Learning Sources

- **Slint Official Docs — Property System:** [`https://slint.dev/docs/rust/slint/src/language/syntax/properties.html`](https://slint.dev/docs/rust/slint/src/language/syntax/properties.html) — Explains property declaration, visibility, and directional rules.
- **Slint Official Docs — Model & Collections:** [`https://slint.dev/docs/rust/slint/src/language/syntax/models.html`](https://slint.dev/docs/rust/slint/src/language/syntax/models.html) — Details how arrays map to `ModelRc<T>` and `VecModel<T>`.
- **Rust ComponentHandle API Reference:** [`https://docs.rs/slint/latest/slint/trait.ComponentHandle.html`](https://docs.rs/slint/latest/slint/trait.ComponentHandle.html) — Explains window instantiation, event loop `.run()`, and `.as_weak()`.

---

## 🧭 2. 30-Second Decision Tree

```text
Title: Property Direction & Model Decision Tree

Question 1: Who creates and mutates the data?
├── Rust alone creates and updates it; UI only displays it
│   └── Prompt: "Chat history, server status, item list"
│       └── SYNTAX: in property <[T]> name; (or in property <T> name;)
│           └── Why: Rust holds single source of truth; UI is read-only projection.
│
├── Human types/drags it in the UI AND Rust needs to inspect/clear it
│   └── Prompt: "Text field, search input, toggle checkbox, slider"
│       └── SYNTAX: in-out property <T> name;
│           └── Why: UI keystrokes push up to Rust; Rust setters push down to UI.
│
└── Data stays 100% inside the UI for animations/hover states (Rust never sees it)
    └── Prompt: "Button hovered flag, drawer open/close animation timer"
        └── SYNTAX: private property <T> name;
            └── Why: Pure internal visual state; zero Rust API clutter.

Question 2: Is the data a collection (list/array)?
├── Static list known at compile time
│   └── SYNTAX: in property <[string]> items: ["A", "B"];
└── Dynamic list mutated by Rust at runtime
    └── SYNTAX: ModelRc<SharedString> backed by Rc<VecModel<SharedString>>
        └── Why: Rc allows shared ownership between Rust state and Slint graphics.
```

---

## ⚖️ 3. Invariant & Operations Matrix

| Property Direction | Host (Rust) Access | UI (Slint) Access | Two-Way Binding (`<=>`) Allowed? | Mutation Invariant |
| :--- | :--- | :--- | :---: | :--- |
| **`in property <T>`** | Read (`get_*`) + Write (`set_*`) | Read Only | ❌ Forbidden | Rust is the producer. Slint cannot mutate directly. |
| **`out property <T>`** | Read Only (`get_*`) | Read + Write | ❌ Forbidden | Slint is the producer (e.g. mouse coordinate). Rust only observes. |
| **`in-out property <T>`** | Read (`get_*`) + Write (`set_*`) | Read + Write | ✅ Required for controls | Bidirectional highway. Both sides can read and write. |
| **`private property <T>`** | ❌ No Rust API generated | Read + Write | Internal only | Encapsulated visual implementation detail. |

---

## 🐍 4. Python & Android Bridge Delta

| Slint & Rust Concept | Android Studio XML/Java Equivalent | CPython / Python GUI Equivalent | Runtime & Memory Reality |
| :--- | :--- | :--- | :--- |
| **`in property <[string]>`** | Read-only `LiveData<List<String>>` feeding `RecyclerView` | Tkinter `StringVar` / PyQt `QStringListModel` read-only | Slint compiles into flat memory array; zero Java/Python GC overhead. |
| **`in-out property <string>`** | Two-way Data Binding `@={viewModel.text}` on `EditText` | PyQt `QLineEdit.textChanged` + `setText()` | Synchronizes typed keystrokes with Rust struct without manual event loops. |
| **`slint::SharedString`** | Java `String` (immutable, UTF-8) | Python `str` (interned/refcounted) | 8-byte reference-counted string buffer; fast O(1) clones across threads/closures. |
| **`slint::VecModel<T>`** | `ArrayList<T>` inside `RecyclerView.Adapter` | Python `list` | Heap-allocated dynamic array; emits row dirty events on `insert()` / `push()`. |
| **`slint::ModelRc<T>`** | Interface `List<T>` passed to Adapter | `Rc<RefCell<list>>` under GIL | Type-erased reference-counted model pointer wrapping the backing array. |

---

## 🧠 5. Physical Metaphor Register

1. **The Glass Bulletin Board (`in property`):** The building manager (Rust) pins announcements behind locked glass; visitors in the lobby (Slint UI) can read the text with their eyes, but their hands cannot alter the paper.
2. **The Handheld Walkie-Talkie (`in-out property`):** The field worker speaks into the microphone (user types keystrokes), and headquarters hears the voice; headquarters can also press the broadcast button to override and clear the channel (Rust resets the field to `""`).
3. **The Shared Clipboard Ring (`Rc<VecModel<T>>`):** A clip ring holding sheets of paper (messages). Rust holds one finger on the ring, and the Slint painter holds another finger on the ring (`Rc` strong count = 2). When Rust slips a new sheet onto the top of the ring, the painter sees it instantly without copying all previous sheets.

---

## 🛠️ 6. Syntax Bank

### Pattern 6.1: Declaring Properties in `.slint`
```slint
export component AppWindow inherits Window {
    // Read-only message history for display
    in property <[string]> messages: ["Initial message"];

    // Bidirectional input text
    in-out property <string> current-input: "";

    // Action signal
    callback send-message();
}
```

### Pattern 6.2: Connecting Live Dynamic Model in Rust (`main.rs`)
```rust
use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    // 1. Create a dynamic, heap-backed vector model
    let messages_vec = vec![
        SharedString::from("Welcome!"),
        SharedString::from("System ready."),
    ];
    let messages_model = Rc::new(VecModel::from(messages_vec));

    // 2. Wrap into a ModelRc and assign to the window
    app.set_messages(ModelRc::from(messages_model.clone()));

    // 3. Insert new message to top of stack (index 0) on click
    let app_weak = app.as_weak();
    let model_clone = messages_model.clone();
    app.on_send_message(move || {
        let Some(app) = app_weak.upgrade() else { return; };
        let new_text = app.get_current_input();
        if !new_text.is_empty() {
            model_clone.insert(0, new_text); // Top of stack!
            app.set_current_input("".into());  // Clear input box
        }
    });

    app.run()
}
```

---

## 🔬 7. Visual Memory Model

```text
Title: Memory Topology: Rc<VecModel<SharedString>> and AppWindow Handle

STACK (main thread)                      HEAP (Shared Buffers)
┌─────────────────────────┐              ┌───────────────────────────────────────┐
│ app: AppWindow          ├─────────────►│ Slint Native Window & GPU Surface     │
│                         │              │ - current-input: SharedString         │
└─────────────────────────┘              │ - messages: ModelRc pointer           │
                                         └──────────────────┬────────────────────┘
┌─────────────────────────┐                                 │ (ModelRc references)
│ messages_model:         │                                 ▼
│ Rc<VecModel<...>>       ├─────────────►┌───────────────────────────────────────┐
│ (strong_count: 2)       │              │ VecModel<SharedString>                │
└─────────────────────────┘              │  ├── [0]: "Latest Message"            │
                                         │  ├── [1]: "Second Message"            │
                                         │  └── [2]: "Oldest Message"            │
                                         └───────────────────────────────────────┘
```

---

## 🛡️ 8. Compiler Diagnostics Table

| Compiler Diagnostic / Panic | Violated Rule | Runtime Bug Prevented | Python / Android Equivalent |
| :--- | :--- | :--- | :--- |
| `error: Cannot assign to an 'in' property` | UI attempted `root.messages = [...]` | Data-race / UI overriding authoritative state | Overwriting a read-only `LiveData` from an XML binding expression. |
| `error[E0599]: no method named set_*` | Called `set_` on a `private` or `out` property | Writing to an unexported or output-only port | Modifying a private Java field without a public setter. |
| `panic: already mutably borrowed` | Holding borrow across event-loop dispatches | Memory corruption from aliased mutation | Modifying a Python list while another thread iterates it without the GIL. |
| `warning: unused variable: app` | Forgetting `let app = ...` keeps handle dead | Window renders but no callbacks ever fire | Initializing an Android `ActivityBinding` without calling `setContentView()`. |

---

## ⚠️ 9. Trap Archive

### Trap 1: The Chained `.new().run()` Unit-Type Trap
- **Assumed:** `let app = AppWindow::new().unwrap().run().unwrap();` stores the window object for setting properties.
- **Reality:** `.run()` blocks until window exit and returns `()`. The variable `app` is the unit type `()`, and the window handle is permanently dropped before any properties are set.
- **10-Second Recall Fix:** Separate the lifecycle into two steps: `let app = AppWindow::new()?;` first, configure all properties/callbacks, then call `app.run()?`.

### Trap 2: String Type Mismatch (`&str` vs `SharedString`)
- **Assumed:** Calling `app.set_current_input("hello");` works directly with a standard Rust string slice.
- **Reality:** Slint properties expect `slint::SharedString` (an atomic reference-counted string); passing `&str` triggers type mismatch error `expected SharedString, found &str`.
- **10-Second Recall Fix:** Call `.into()` or `SharedString::from("hello")` to convert any `&str` or `String`.

### Trap 3: The Closure Self-Cycle Leak
- **Assumed:** Moving `app` directly into `app.on_send_message(move || app.set_...)` is clean and safe.
- **Reality:** The window owns the callback closure, and the closure owns the window handle (`app`), forming an `Rc` strong reference cycle that leaks the entire window memory forever.
- **10-Second Recall Fix:** Downgrade to weak before moving: `let app_weak = app.as_weak();` and upgrade inside: `let Some(app) = app_weak.upgrade() else { return; };`.

### Trap 4: The Unplugged Model Cable Trap
- **Assumed:** Creating a `VecModel` in Rust and mutating it automatically updates the window on screen.
- **Reality:** The window renders its default internal data until you explicitly plug your model in; mutating an unattached `message_stack` clears the input box but leaves the screen frozen.
- **10-Second Recall Fix:** Always plug the cable in during setup: `app.set_messages(ModelRc::from(message_stack.clone()));`.

### Trap 5: The "move" Keyword Does Not Mean "FnOnce"
- **Assumed:** Passing variables via `move ||` consumes them and prevents subsequent button clicks.
- **Reality:** `move` only packs variables into the closure struct at creation time. Calling `.insert()` merely borrows `message_stack` via `&`, so the closure implements `FnMut` and survives for millions of clicks.
- **10-Second Recall Fix:** `move` puts items *into* the closure; only moving items *out of* the closure makes it `FnOnce`.

### Trap 6: The ScrollView Squishing vs. ListView Trap
- **Assumed:** Nesting `VerticalLayout` inside a `ScrollView` creates an Android-like scrollable list.
- **Reality:** `VerticalLayout` calculates its height from the parent and squishes all message cards into paper-thin slices rather than overflowing and scrolling.
- **10-Second Recall Fix:** For dynamic collections, always use native `ListView`, which preserves natural item heights and activates smooth scrolling.

### Trap 7: The Short-Word Column Collapse (Card Geometry)
- **Assumed:** Wrapping text inside an aligned horizontal row automatically fits bubble width neatly around any content.
- **Reality:** In Slint, text with `wrap: TextWrap.word-wrap` inside an unconstrained layout collapses to the width of the shortest word; short words like "hi" compress the bubble into a 10px-wide, 100px-tall vertical pillar.
- **10-Second Recall Fix:** Set explicit fixed width (e.g., `width: 320px;`) and dynamic height on the card with `horizontal-alignment: left`.

---

## ⏳ 10. Deferred Concepts (Manifest Boundary)

- `⏳ BANNED IN TOPIC 07 — Topic 08 (Concurrency):` Spawning background threads with `std::thread::spawn` and updating properties from worker threads via `slint::invoke_from_event_loop()`.
- `⏳ BANNED IN TOPIC 07 — Topic 09 (CLI & Filesystem):` Persisting chat history to disk using `std::fs` or serializing messages with `serde_json`.
- `⏳ BANNED IN TOPIC 07 — Topic 11 (Networking & Async):` Streaming LLM token responses over network sockets using Tokio and async channels.

---

## 📈 11. Family Learning Progression

```text
Complexity ▲
           │                                      [04: Custom Enums & Structs]
           │                                 ┌───► Typed complex payloads
           │                        [03: ModelRc & VecModel]
           │                   ┌───► Dynamic list insertion
           │         [02: in-out Two-Way Binding]
           │    ┌───► LineEdit <=> current-input
           │ [01: in Property]
           └───► Read-only values (Display)
           ─────────────────────────────────────────────────────────────► Phase
```

---

## 📎 12. Concept Notes Directory

- [`01_chat_stack_ui_architecture.md`](01_chat_stack_ui_architecture.md) — Comprehensive architecture report for the Chat Stack container layout, stretch geometry, and Android mental bridge.
- [`02_whatsapp_bubble_layout.md`](02_whatsapp_bubble_layout.md) — Tier 2 WhatsApp-Style Bubble Layout & Card Geometry Report.
- [`README.md`](README.md) — Active UI domain master registry.

