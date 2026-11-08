# 🤖 AI Telemetry Bubble & Sentinel Console (`ui/chat_bubbles/ai-bubble.slint`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`ui/chat_bubbles/ai-bubble.slint`](file:///home/pirate/development/ai-workflow-gui/ui/chat_bubbles/ai-bubble.slint) |
| **Document Path** | `reports/docs/ui/05_ai_telemetry_bubble.md` |
| **Domain** | UI Architecture & Real-Time Telemetry Cards |
| **Target Persona** | Junior Android Developer (`Custom ViewHolder`, `ValueAnimator`, `StateFlow`, `TransitionManager`) |
| **Android Architectural Anchor** | Advanced custom `ViewHolder` with dynamic state animations and live StateFlow bindings |
| **Verification Mode** | Codebase Verified (`ui/chat_bubbles/ai-bubble.slint`, `src/lib.rs`, `src/main.rs`) |
| **Confidence** | VERIFIED |

> **TL;DR:** `ui/chat_bubbles/ai-bubble.slint` is the high-density telemetry engine of the chat stream. It renders a 430px Cyber-Slate card featuring a 4-tier live status strip (pulsing amber orb, task progress, dual RPM/API velocity, error badge), an animated hover-expanding `|>` drawer trigger, and a multi-line word-wrapped Sentinel console.

---

## 🔬 1. Visual Card Architecture

In Android Studio, rendering a chat bubble that updates live metrics (such as requests per minute, tool execution progress, and animated status lights) usually requires a custom `ConstraintLayout` inside a `ViewHolder`, observed via Kotlin Coroutine `StateFlow` and updated with `ValueAnimator`.

In Slint, `AiBubble` achieves this entirely declaratively with zero CPU thread contention:

```text
+-----------------------------------------------------------------------------+
|                          AiBubble VISUAL ANATOMY                            |
+-----------------------------------------------------------------------------+
|  width: 430px, border-radius: 12px, background: #161f30 -> #0f172a          |
|  border-color: Amber (is-generating) / Emerald (completed) / Red (errors)   |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | 1. TOP TELEMETRY STRIP (height: 32px)                                 |  |
|  |  [●] [Task 1/3]  [⚡ 45 RPM • 4 Reqs]  [ERR: 1]        [|> Inspect Logs] |  |
|  |  Status   Progress    API Throughput      Alert           Hover Trigger   |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | 2. SENDER & TICKET IDENTIFIER                                         |  |
|  | ColdWind AI                                     #development-7a1b...  |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | 3. SENTINEL LIVE CONSOLE (if msg.is-generating == true)                |  |
|  |  || LIVE AGENT TELEMETRY & QUOTES                         45 req/min   |  |
|  |  || "Formulating query for SQL knowledge base. Synthesizing tool       |  |
|  |  || parameters across multi-hop reasoning graph..."                    |  |
|  +-----------------------------------------------------------------------+  |
|  | 3b. FINAL RESPONSE TEXT (if msg.is-generating == false)                |  |
|  |  "Based on the analysis, here is the complete execution report..."     |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | 4. STATUS & LATENCY FOOTER                                            |  |
|  | ● BROADCASTING TELEMETRY                                      1240 ms |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+
```

### The Android ↔ Slint Mental Bridge

| Concept | Android Studio Equivalent | Slint (`ai-bubble.slint`) Equivalent | Architectural Purpose |
| :--- | :--- | :--- | :--- |
| **Card Boundary** | `CardView` with reactive `setStrokeColor()` | `Rectangle` with conditional `border-color` and `animate border-color` | Visually communicates system state (Amber = Active, Emerald = Done, Red = Error). |
| **Live Metric Pills** | `Chip` or custom badge XML drawable | Nested `Rectangle` with `border-radius: 4px` | Displays micro-telemetry (RPM, task counter, error alerts) compactly. |
| **Hover Button** | `View.animate().scaleX(...)` / `ObjectAnimator` | `Rectangle` with `animate width, background { duration: 180ms; }` | Expands drawer button smoothly from 28px to 132px on cursor hover. |
| **Status Switch** | `viewStub.inflate()` or `view.visibility` | `if root.msg.is-generating: ...` | Toggles between live telemetry quote box and final markdown transcript. |
| **In-Place Update** | `adapter.notifyItemChanged(position)` | `VecModel::set_row_data(idx, updated_msg)` | Mutates the bubble directly on the GPU without rebuilding the list. |

---

## 🎛️ 2. Section-by-Section Code Walkthrough

### 2.1 The Dynamic Status Glow Border

```slint
width: 430px;
border-radius: 12px;
border-width: 1px;
border-color: root.msg.is-generating ? #f59e0b99 :
    (root.msg.fail-count > 0 ? #ef444488 : #26334d);
background: @linear-gradient(180deg, #161f30 0%, #0f172a 100%);
clip: true;

animate border-color {
    duration: 220ms;
    easing: ease-out;
}
```

- **Reactive Border Calculation:**
  - While generating (`root.msg.is-generating` is `true`), the border glows in **Amber** (`#f59e0b99`).
  - If finished with errors (`fail-count > 0`), the border transitions to **Crimson Red** (`#ef444488`).
  - When finished cleanly, it returns to dark **Slate Navy** (`#26334d`).
- **`animate border-color`:** Slint interpolates the color over 220 milliseconds. When a workflow finishes, the border smoothly shifts from bright amber to calm slate with zero pop or jarring flash.

---

### 2.2 The Top Telemetry Strip & Metric Pills

```slint
HorizontalLayout {
    padding-left: 9px;
    padding-right: 4px;
    spacing: 6px;

    // Live Status Dot (Amber = Generating, Red = Error, Emerald = Completed)
    Rectangle {
        width: 8px;
        height: 8px;
        border-radius: 4px;
        background: root.msg.is-generating ? #f59e0b :
            (root.msg.fail-count > 0 ? #ef4444 : #10b981);
        y: (parent.height - self.height) / 2;
        animate background { duration: 200ms; }
    }
    // ...
```

The strip organizes four distinct telemetry metrics side-by-side:

| Pill | Data Source | Visual Presentation | Meaning |
| :--- | :--- | :--- | :--- |
| **Status Orb** | `is-generating`, `fail-count` | 8px circular dot | Amber = Working, Red = Error, Emerald = Done. |
| **Task Progress** | `msg.task-progress` | Slate pill with white bold text (`[1/3]`) | Displays subtask index out of total workflow plan. |
| **Dual API Velocity** | `msg.rpm-count`, `msg.api-count` | Dark amber pill with electric yellow text | Shows real-time 60s sliding window RPM + total lifetime turn requests. |
| **Fault Counter** | `msg.fail-count` | Crimson alert pill (`ERR: 1`) | Only rendered if errors > 0 (`if root.msg.fail-count > 0`). |

---

### 2.3 The Hover-Expanding `|>` Drawer Button

```slint
slide-btn := Rectangle {
    horizontal-stretch: 0;
    width: btn-touch.has-hover ? 132px : 28px;
    height: 22px;
    y: (parent.height - self.height) / 2;
    background: btn-touch.has-hover ? #6366f1 : #1e293b;
    border-radius: 5px;
    border-width: 1px;
    border-color: btn-touch.has-hover ? #818cf8 : #334155;

    animate width, background, border-color {
        duration: 180ms;
        easing: ease-out;
    }

    HorizontalLayout {
        alignment: center;
        Text {
            text: btn-touch.has-hover
                ? (root.msg.is-generating ? "|> Live Console" : "|> Inspect Logs")
                : "|>";
            color: btn-touch.has-hover ? #ffffff : #818cf8;
            font-size: 10px;
            font-weight: 700;
            vertical-alignment: center;
        }
    }

    btn-touch := TouchArea {
        mouse-cursor: pointer;
        clicked => {
            root.inspect-ticket(root.msg.ticket-id);
        }
    }
}
```

#### How the Hover Expansion Works

1. In rest state, `btn-touch.has-hover` is `false`. The button is a compact 28px square showing `|>`.
2. When the user moves their cursor over the button, `has-hover` becomes `true`.
3. Slint automatically triggers the animation block:
   ```slint
   animate width, background, border-color { duration: 180ms; easing: ease-out; }
   ```
4. Over 180 milliseconds, the button smoothly expands from **28px to 132px**, morphs from muted slate (`#1e293b`) to vibrant Indigo (`#6366f1`), and reveals the text `|> Live Console` or `|> Inspect Logs`.
5. Clicking the button fires `root.inspect-ticket(root.msg.ticket-id)`, opening the 75% slide-in drawer.

---

### 2.4 The Multi-Line Sentinel Live Console

```slint
if root.msg.is-generating: Rectangle {
    min-height: 68px;
    background: #0b111e;
    border-radius: 8px;
    border-width: 1px;
    border-color: #f59e0b66;

    HorizontalLayout {
        padding: 10px;
        spacing: 10px;
        alignment: stretch;

        // Left Amber Accent Bar
        Rectangle {
            width: 4px;
            border-radius: 2px;
            background: #f59e0b;
        }

        VerticalLayout {
            horizontal-stretch: 1;
            spacing: 4px;
            alignment: start;

            HorizontalLayout {
                alignment: space-between;
                Text {
                    text: "LIVE AGENT TELEMETRY & QUOTES";
                    color: #fbbf24;
                    font-size: 10px;
                    font-weight: 700;
                }
                Text {
                    text: root.msg.rpm-count + " req/min";
                    color: #f59e0b;
                    font-size: 9.5px;
                }
            }

            // Multi-line Word-Wrapped Live Quote / Action Summary (No Elide!)
            Text {
                text: root.msg.live-summary;
                color: #e2e8f0;
                font-size: 12px;
                wrap: TextWrap.word-wrap;
                horizontal-alignment: left;
            }
        }
    }
}
```

#### Why Word-Wrap Is Essential Here

In older versions of the UI, status text used `overflow: elide` (single-line truncation with `...`). When the agent produced a funny quote or executed a multi-parameter tool call (e.g., `"Executing database query across 12 partitions..."`), the text was cut off after 20 characters.

By replacing `overflow: elide` with **`wrap: TextWrap.word-wrap`** and setting `min-height: 68px`, the box dynamically expands vertically to accommodate two, three, or four lines of text. The user can read the complete agent thought stream directly inside the chat feed without even having to open the drawer!

---

## ⚡ 3. How Rust Drives In-Place Live Updates

In Android, if you want to update one item in a `RecyclerView`, you do:
```kotlin
adapter.notifyItemChanged(lastIndex)
```

In Rust (`src/main.rs`), Thread 3 consumes log entries and updates the active AI message directly on Thread 0 via `slint::invoke_from_event_loop`:

```rust
// Runs inside src/main.rs on Thread 3:
slint::invoke_from_event_loop(move || {
    if let Some(app) = ui_weak.upgrade() {
        let model = app.get_messages();
        let row_count = model.row_count();
        if row_count > 0 {
            let last_idx = row_count - 1;
            if let Some(mut msg) = model.row_data(last_idx) {
                if msg.is_generating {
                    // Update telemetry fields:
                    msg.rpm_count = telemetry.rpm_count;
                    msg.api_count = telemetry.total_requests;
                    msg.task_progress = telemetry.format_progress().into();
                    msg.live_summary = quote.into();

                    // Push back to model in-place (ZERO FLICKER):
                    model.set_row_data(last_idx, msg);
                }
            }
        }
    }
});
```

Because `set_row_data` updates only the single row's memory slot, Slint's GPU renderer re-renders only the changed text nodes. The list does not re-scroll, and the cards never flicker.

---

## ⚠️ 4. Common Traps & Pitfalls for Android Developers

### Trap 1: Hover Collisions on Expanding Elements
- **The Mistake:** Placing an expanding button next to variable-width text without setting `alignment: space-between` or fixed constraints.
- **The Reality:** When the button expands from 28px to 132px, it will push or squish neighboring text if they are in the same flexible layout.
- **The Fix:** In `AiBubble`, the left telemetry pills are grouped into their own `HorizontalLayout { horizontal-stretch: 1; alignment: start; }` and the button has `horizontal-stretch: 0`. This guarantees the button expands smoothly into empty space without crushing any metric badges.

### Trap 2: Mutating Primary Keys on In-Flight Messages
- **The Mistake:** Resetting `msg.ticket-id = "pending"` during live updates.
- **The Reality:** If the user clicks `|> Inspect Logs` while the turn is active, a reset primary key breaks the lookup and displays empty logs.
- **The Fix:** The ticket ID is stamped once on Thread 0 and remains immutable for the lifetime of the bubble.

---

## ❓ 5. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Why is `width` set to 430px for `AiBubble` while `UserBubble` is 340px? | AI responses contain rich telemetry strips, four metric badges, and multi-line agent quotes that require extra horizontal breathing room, whereas user prompts are simple conversational text. | 2026-11-04 |
| Q2 | Does `animate width` on the `slide-btn` run on the CPU or GPU? | Slint computes property interpolation during its frame clock and submits updated vertex coordinates to the backend renderer (OpenGL / Vulkan / Software), guaranteeing 60 FPS animation. | 2026-11-04 |
| Q3 | When does `is-generating` become false? | When Python core finishes the turn and Thread 1 receives the final `TicketOutResult`. Rust sets `msg.is_generating = false`, populates `msg.text` with the final transcript, and updates the row. | 2026-11-04 |
