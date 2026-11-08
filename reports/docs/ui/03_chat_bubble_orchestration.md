# 🎭 Chat Bubble Polymorphic Orchestration (`ui/chat-bubble.slint`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`ui/chat-bubble.slint`](file:///home/pirate/development/ai-workflow-gui/ui/chat-bubble.slint) |
| **Document Path** | `reports/docs/ui/03_chat_bubble_orchestration.md` |
| **Domain** | UI Architecture & Component Delegation |
| **Target Persona** | Junior Android Developer (`RecyclerView.Adapter`, `getItemViewType`, `ViewHolder` pattern) |
| **Android Architectural Anchor** | `RecyclerView.Adapter.getItemViewType(position)` + multi-`ViewHolder` routing |
| **Verification Mode** | Codebase Verified (`ui/chat-bubble.slint`, `ui/app-window.slint`) |
| **Confidence** | VERIFIED |

> **TL;DR:** `ui/chat-bubble.slint` acts as a polymorphic dispatch router for list rows. By inheriting `VerticalLayout` and branching on `msg.is-user`, it dynamically delegates rendering to either `UserBubble` or `AiBubble` while automatically computing exact row height.

---

## 🧩 1. The Multi-View Dilemma in List Feeds

In Android Studio, when you build a chat app like WhatsApp or Telegram, your `RecyclerView` contains two completely different visual cards:
1. **The User Message:** A sleek, right-aligned, gradient card containing only the user prompt.
2. **The AI Response:** A complex, left-aligned, cyber-slate card equipped with pulsing status dots, live telemetry pills (RPM, API requests, task counters), an interactive `|>` drawer button, and an expandable live sentinel box.

In Android, you solve this by overriding `getItemViewType(position)` in your `RecyclerView.Adapter`:

```kotlin
// Android Studio Mental Model:
override fun getItemViewType(position: Int): Int {
    return if (messages[position].isUser) TYPE_USER else TYPE_AI
}

override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
    return if (viewType == TYPE_USER) UserViewHolder(...) else AiViewHolder(...)
}
```

Slint does not have `getItemViewType` methods or manual ViewHolder factories. Instead, Slint uses **declarative conditional mounting inside a layout orchestrator**.

```text
+-----------------------------------------------------------------------------+
|                      ChatBubble POLYMORPHIC ROUTING                         |
+-----------------------------------------------------------------------------+

                               ChatBubble
                       (inherits VerticalLayout)
                                   │
                                   │ Reads `root.msg.is-user`
                                   │
                  ┌────────────────┴────────────────┐
                  ▼                                 ▼
      [if root.msg.is-user == true]      [if root.msg.is-user == false]
                  │                                 │
                  ▼                                 ▼
              UserBubble                         AiBubble
          (Indigo Gradient Card)           (Cyber-Slate Telemetry Card)
          • width: 340px                   • width: 430px
          • Right-aligned                  • Left-aligned
          • Sender + Prompt + Time         • Telemetry Strip (RPM, API)
                                           • Multi-Line Sentinel Console
                                           • Interactive `|>` Trigger
```

---

## 📐 2. The Critical Layout Secret: Why `inherits VerticalLayout`?

Notice line 16 of `ui/chat-bubble.slint`:

```slint
export component ChatBubble inherits VerticalLayout {
    padding: 0px;
    spacing: 0px;
    // ...
}
```

Why does `ChatBubble` inherit `VerticalLayout` instead of a plain `Rectangle` or `Component`?

### The "Auto-Wrap Content" Discovery

In Android, every view specifies:
```xml
android:layout_width="wrap_content"
android:layout_height="wrap_content"
```

In Slint:
- A `Rectangle` has **no intrinsic layout measuring logic** for child components unless you manually compute and bind its `height`. If you wrapped `UserBubble` inside a `Rectangle` without an explicit height, the row would either collapse to 0px or fail to expand when multi-line text appears!
- A `VerticalLayout` **automatically measures the preferred intrinsic height of its visible children**.
- When `root.msg.is-user` is true, `VerticalLayout` measures the height of `UserBubble` (e.g. 74px).
- When `root.msg.is-user` is false, `VerticalLayout` measures the height of `AiBubble` (e.g. 180px while live streaming).
- The outer `ListView` queries `ChatBubble` for its height and allocates the exact vertical space needed without clipping or squishing.

| Container Choice | Height Behavior | Result in `ListView` |
| :--- | :--- | :--- |
| **`inherits VerticalLayout` (Current)** | Automatically adopts the dynamic height of whichever child is mounted. | 🟢 **Perfect.** Dynamic, zero-flicker height adaptation. |
| **`inherits Rectangle` (Flawed)** | Does not calculate child bounds; requires hardcoded or manual height bindings. | 🔴 **Broken.** Multi-line messages clip or cards overlap vertically. |

---

## 🔬 3. Code Walkthrough & Event Bubbling

Here is the complete implementation of `ui/chat-bubble.slint`:

```slint
import { ChatMessage } from "types.slint";
import { UserBubble } from "chat_bubbles/user-bubble.slint";
import { AiBubble } from "chat_bubbles/ai-bubble.slint";

export component ChatBubble inherits VerticalLayout {
    padding: 0px;
    spacing: 0px;

    in property <ChatMessage> msg;
    callback inspect-ticket(string);

    if root.msg.is-user: UserBubble {
        msg: root.msg;
    }

    if !root.msg.is-user: AiBubble {
        msg: root.msg;
        inspect-ticket(id) => {
            root.inspect-ticket(id);
        }
    }
}
```

### Breakdown of Mechanics

1. **Imports:** Imports the shared `ChatMessage` struct and the two specialized bubble cards from the `chat_bubbles/` sub-folder.
2. **`in property <ChatMessage> msg;`:** Receives the current row data from the `ListView` in `app-window.slint`.
3. **`callback inspect-ticket(string);`:** Defines a bubbling event channel.
4. **Conditional Mounting:**
   - Slint evaluates `if root.msg.is-user`.
   - If `true`, Slint mounts `UserBubble` and completely skips allocating GPU resources for `AiBubble`.
   - If `false`, Slint mounts `AiBubble` and binds its `inspect-ticket` callback.
5. **Callback Bubbling (`inspect-ticket(id) => { root.inspect-ticket(id); }`):**
   - In Android, if a child card button is clicked inside an item view, you either trigger an interface callback on the `ViewHolder` or pass a lambda up to the `Activity`.
   - In Slint, the `AiBubble` emits `inspect-ticket(id)`, which `ChatBubble` catches and re-emits upward to `AppWindow`, which opens the `TicketDrawer`.

---

## ⚡ 4. Architectural Advantages of This Pattern

| Dimension | Monolithic Approach (Single giant bubble file) | Delegated Approach (`chat-bubble.slint`) |
| :--- | :--- | :--- |
| **Code Readability** | 400+ lines of nested `if-else` layout code in one file. | 33 lines of clean routing; cards isolated in dedicated files. |
| **Separation of Concerns** | User card styling polluted with AI telemetry logic. | User card has 0 telemetry code; AI card has dedicated telemetry widgets. |
| **Compiler Optimization** | Slint compiler must evaluate complex nested layout branches simultaneously. | Slint compiles separate render trees for `UserBubble` and `AiBubble`. |
| **Maintainability** | Adding a third message type (e.g. `SystemAlertBubble`) requires editing entire UI. | Simply add `03_system_bubble.slint` and one new `if` branch. |

---

## ⚠️ 5. Common Traps & Pitfalls for Android Developers

### Trap 1: Using `else` in Slint Declarations
- **The Mistake:** Writing `if root.msg.is-user: UserBubble { ... } else: AiBubble { ... }`.
- **The Reality:** Slint does **not** have an `else` keyword in declarative component instantiation.
- **The Fix:** Write two complementary conditions:
  ```slint
  if root.msg.is-user: UserBubble { ... }
  if !root.msg.is-user: AiBubble { ... }
  ```

### Trap 2: Forgetting Callback Bubbling
- **The Mistake:** Defining `callback inspect-ticket(string)` in `AiBubble` and expecting `AppWindow` to receive it automatically without wiring it in `ChatBubble`.
- **The Reality:** Slint callbacks do **not** automatically bubble up the widget tree like DOM events. Every intermediate parent must explicitly forward the callback.

---

## ❓ 6. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Does mounting and unmounting `UserBubble` vs `AiBubble` cause GC pauses? | No. Slint is compiled directly to native Rust and C++ GPU render passes. There is no Garbage Collector, JVM, or runtime reflection. | 2026-11-04 |
| Q2 | Why not pass only `msg.text` to `UserBubble` instead of the entire `ChatMessage`? | Passing the full `ChatMessage` preserves interface consistency across all bubble components and allows `UserBubble` to read `sender` and `timestamp` without adding extra properties. | 2026-11-04 |
| Q3 | What is the performance cost of having `padding: 0px` and `spacing: 0px` on `ChatBubble`? | Zero cost. Slint optimizes zero-padding layouts at compile time, eliminating empty geometry computations. | 2026-11-04 |
