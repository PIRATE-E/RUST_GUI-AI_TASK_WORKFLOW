# 🗄️ 75% Widescreen Telemetry Drawer (`ui/ticket-drawer.slint`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`ui/ticket-drawer.slint`](file:///home/pirate/development/ai-workflow-gui/ui/ticket-drawer.slint) |
| **Document Path** | `reports/docs/ui/06_telemetry_drawer.md` |
| **Domain** | UI Architecture & Slide-Over Modal Inspection |
| **Target Persona** | Junior Android Developer (`BottomSheetDialogFragment`, `NavigationDrawer`, Scrim, Touch Interception) |
| **Android Architectural Anchor** | `BottomSheetDialogFragment` / `NavigationDrawer` with modal scrim and touch consumption |
| **Verification Mode** | Codebase Verified (`ui/ticket-drawer.slint`, `src/lib.rs`, `src/main.rs`) |
| **Confidence** | VERIFIED |

> **TL;DR:** `ui/ticket-drawer.slint` is a 75%-width slide-in telemetry console. It slides smoothly from the right edge with a 220ms GPU animation, features a touch-absorbing modal backdrop dimmer, renders a widescreen untruncated live quote hero console, and provides a 3-card reactive KPI dashboard alongside a severity-tagged execution log feed.

---

## 🏛️ 1. Modal Slide-Over Architecture

In Android Studio, when you want to show detailed diagnostics without navigating away from the chat screen, you display an expanded `BottomSheetDialogFragment` or a custom `NavigationDrawer`. Android automatically places a dark translucent "scrim" (dimmer) behind the dialog and dismisses it when the user taps outside.

In Slint, `TicketDrawer` implements this exact desktop slide-over pattern:

```text
+-----------------------------------------------------------------------------+
|                         AppWindow CANVAS (100% WIDTH)                       |
+-----------------------------------------------------------------------------+
|  LEFT 25%: MODAL SCRIM BACKDROP         | RIGHT 75%: SLIDING TELEMETRY PANEL |
|  background: #00000099 (Translucent)    | x: root.is-open ? 25% : 100%       |
|  Clicking here calls `close-drawer()`   | width: root.width * 0.75           |
|                                         |                                    |
|                                         | +--------------------------------+ |
|                                         | | A. Header: Ticket ID & State   | |
|                                         | +--------------------------------+ |
|                                         | | B. Origin User Prompt Anchor   | |
|                                         | +--------------------------------+ |
|                                         | | C. Widescreen Hero Console     | |
|                                         | |    (Untruncated Live Quotes)   | |
|                                         | +--------------------------------+ |
|                                         | | D. 3-Card Reactive KPI Strip   | |
|                                         | |    [Progress] [RPM/API] [Fault]| |
|                                         | +--------------------------------+ |
|                                         | | E. Execution Log Stream        | |
|                                         | |    (Virtualized ListView)      | |
|                                         | +--------------------------------+ |
+-----------------------------------------------------------------------------+
```

### The Android ↔ Slint Mental Bridge

| Concept | Android Studio Equivalent | Slint (`ticket-drawer.slint`) Equivalent | Architectural Purpose |
| :--- | :--- | :--- | :--- |
| **Slide-Over Panel** | `BottomSheetDialogFragment` / `DrawerLayout` | `panel := Rectangle { width: root.width * 0.75; ... }` | Provides ample desktop widescreen canvas for deep telemetry inspection. |
| **Backdrop Dimmer** | Window Scrim (`android:backgroundDimColor`) | `Rectangle { background: root.is-open ? #00000088 : #00000000; }` | Dims background chat feed when drawer opens instantly without full-screen redraw thrashing. |
| **Dismiss on Tap Outside** | `setCanceledOnTouchOutside(true)` | Left `TouchArea` calling `root.close-drawer()` | Catches clicks on the backdrop and closes the drawer. |
| **Click Swallowing** | `android:clickable="true"` | Empty `TouchArea { }` inside panel | Swallows mouse clicks inside the drawer so they don't trigger backdrop dismissal. |
| **Slide Transition** | `ObjectAnimator.ofFloat(view, "translationX", ...)` | `animate x { duration: 180ms; easing: cubic-bezier(...); }` | Translates the panel from off-screen (`x = width`) to on-screen (`x = 0.25 * width`). |

---

## 🎛️ 2. Section-by-Section Code Walkthrough

### 2.1 The Coordinate-Based Slide Animation

Notice lines 48–63 of `ui/ticket-drawer.slint`:

```slint
panel := Rectangle {
    cache-rendering-hint: true;
    width: root.width * 0.75;
    height: 100%;
    x: root.is-open ? root.width * 0.25 : root.width;
    background: #090d16;
    border-width: 1px;
    border-color: root.active-ticket.is-generating ? #f59e0b88 : #334155;

    animate x, border-color {
        duration: 180ms;
        easing: cubic-bezier(0.16, 1, 0.3, 1);
    }
```
```

#### How the Slide Animation Works

1. **Closed State (`is-open: false`):**
   - `x` is set to `root.width` (the far right edge of the window).
   - The panel is positioned completely off-screen, invisible to the user.
   - The backdrop color is `#00000000` (100% transparent).
2. **Open State (`is-open: true`):**
   - `x` transitions to `root.width * 0.25`.
   - The panel covers the right 75% of the screen.
   - `animate x` drives the motion smoothly across 220ms using an `ease-out` velocity curve.
   - The backdrop color animates to `#00000099` (60% dark translucent scrim).

---

### 2.2 Event Swallowing: The Empty `TouchArea` Trick

In Android, if you put a dialog over an activity, touch events on the dialog do not fall through to the activity underneath because views consume touch events.

In Slint, mouse clicks pass through rectangles unless an explicit `TouchArea` intercepts them! Notice lines 41–65:

```slint
// 1. Catch clicks on the dimmer area to close the drawer:
if root.is-open: TouchArea {
    clicked => {
        root.close-drawer();
    }
}

panel := Rectangle {
    // ...
    // 2. Prevent clicks inside the drawer panel from bubbling to the backdrop:
    TouchArea { }
    // ...
```

- If line 64 (`TouchArea { }`) did not exist, clicking anywhere inside the drawer panel (such as scrolling a log or reading text) would pass through the panel, hit the background `TouchArea`, and accidentally close the drawer!
- The empty `TouchArea { }` acts as a **touch consumer** (equivalent to `android:clickable="true"` in Android XML).

---

### 2.3 Widescreen Hero Console (Untruncated Streaming)

```slint
Rectangle {
    min-height: 82px;
    border-radius: 10px;
    border-width: 1px;
    background: root.active-ticket.is-generating
        ? @linear-gradient(135deg, #1c1917 0%, #0f172a 100%)
        : @linear-gradient(135deg, #064e3b33 0%, #0f172a 100%);
    border-color: root.active-ticket.is-generating ? #f59e0b88 : #10b98166;

    // ...
    Text {
        text: root.active-ticket.live-summary;
        color: #f8fafc;
        font-size: 13px;
        font-weight: 600;
        wrap: TextWrap.word-wrap;
        horizontal-alignment: left;
    }
}
```

- When the workflow is running, the hero box glows in **Amber** with the heading `📡 LIVE TELEMETRY & AGENT QUOTES (UNTRUNCATED)` and streaming indicator `Streaming @ 60 FPS`.
- When completed, the box smoothly transitions via `animate border-color { duration: 250ms; }` into **Emerald Green** (`#10b98166`) with the heading `✓ FINAL WORKFLOW EXECUTION STATUS` and displays `Total Turn Time`.
- Because the panel is 75% of the desktop window (~570px wide), complete agent quotes and multi-step tool calls are displayed without elision or truncation.

---

### 2.4 The 3-Card KPI Strip

```slint
HorizontalLayout {
    spacing: 10px;
    height: 64px;

    // KPI 1: Workflow Progress
    // KPI 2: Dual API Velocity (RPM + Total Requests)
    // KPI 3: Pipeline Health & Fault Counter
}
```

The drawer features three real-time telemetry cards side-by-side:

| KPI Card | Metric Displayed | Normal State | Warning / Error State |
| :--- | :--- | :--- | :--- |
| **Workflow Progress** | `active-ticket.task-progress` | Cyan `#38bdf8` (`Task [1/3]`) | Displays subtask index dynamically. |
| **API Throughput** | `rpm-count` + `api-count` | Slate `#e2e8f0` (`0 RPM • 4 Reqs`) | Pulsing Amber `#fbbf24` while generating (`45 RPM • 4 Reqs`). |
| **Pipeline Health** | `active-ticket.fail-count` | Emerald `#34d399` (`0 Faults • Healthy`) | Crimson `#f87171` (`⚠ 2 API Failures`) with red border glow. |

---

### 2.5 Execution Log Stream & Phase A Standby Card

```slint
if root.logs.length == 0: Rectangle {
    // Shows Informative Standby Card
    Text {
        text: root.active-ticket.is-generating
            ? "⚡ Live Telemetry Active — Streaming Turn Telemetry @ 60 FPS"
            : "✓ Turn Finished — Detailed Log Graph Buffering Queued for Phase B";
    }
}

if root.logs.length > 0: ListView {
    for log in root.logs: VerticalLayout {
        // Renders severity-coded log card (ERROR, WARNING, INFO)
    }
}
```

- **Phase A Standby Card:** In the current phase of development (Phase A), live metrics (RPM, API requests, quotes) stream directly into the Hero Console and KPI cards. If the raw log buffer is empty (`root.logs.length == 0`), the drawer displays a helpful standby card rather than an ugly blank void.
- **Phase B Ready:** In Phase B, Rust will populate `active-ticket-logs` with `DrawerLogItem` structs, which will automatically mount the virtualized `ListView` with zero UI code changes required!

---

## ⚡ 3. How Rust Drives the Drawer

When the user clicks `|>` on any chat bubble:
1. `ChatBubble` forwards `inspect-ticket(ticket_id)` to `AppWindow`.
2. `AppWindow` executes:
   ```slint
   root.active-ticket = msg;
   root.drawer-open = true;
   root.inspect-ticket(id);
   ```
3. In `src/main.rs`, Rust's callback receives `ticket_id`:
   ```rust
   let ui_weak = app.as_weak();
   app.on_inspect_ticket(move |ticket_id| {
       // Rust can query disk logs or in-memory ring buffers
       // and populate active-ticket-logs on the UI thread!
   });
   ```

Because `active-ticket` is two-way bound (`in-out property <ChatMessage> active-ticket`), any live telemetry update sent to the chat bubble by Thread 3 automatically reflects inside the drawer in real time!

---

## ⚠️ 4. Common Traps & Pitfalls for Android Developers

### Trap 1: Hardcoding Absolute Pixel Widths on Drawers
- **The Mistake:** Writing `width: 500px;` for the drawer panel.
- **The Reality:** On large monitors (1440p or 4K), a 500px drawer looks like a tiny strip. On small windows, it overflows.
- **The Fix:** Use proportional sizing: `width: root.width * 0.75;` and `x: root.is-open ? root.width * 0.25 : root.width;`. This ensures the drawer is always exactly 75% of the window width, regardless of resolution.

### Trap 2: Missing the Inner TouchArea (Accidental Dismissal)
- **The Mistake:** Omitting `TouchArea { }` inside the drawer panel.
- **The Reality:** Clicking on any log item or button inside the drawer will pass through to the backdrop dimmer and close the drawer unexpectedly.

---

## ❓ 5. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Does the 75% drawer block keyboard input to the main chat box when open? | Yes. The backdrop dimmer covers the main chat feed and input dock, preventing accidental prompt submissions while inspecting logs. | 2026-11-04 |
| Q2 | Can the drawer be closed with the Escape key? | Currently it closes via the `✕` button or clicking the left 25% scrim. A `FocusScope` with `key-pressed` handler can be added to support the Escape key. | 2026-11-04 |
| Q3 | How does the drawer know the previous user prompt? | `AppWindow` checks `root.messages[idx - 1].is-user` when `inspect-ticket` is triggered and automatically copies the user prompt to `active-ticket-user-prompt`. | 2026-11-04 |
