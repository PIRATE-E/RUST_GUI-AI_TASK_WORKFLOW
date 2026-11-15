# 💬 User Bubble Presentation Component (`ui/chat_bubbles/user-bubble.slint`)

| Metric | Specification |
| :--- | :--- |
| **File Under Documentation** | [`ui/chat_bubbles/user-bubble.slint`](../../../ui/chat_bubbles/user-bubble.slint) |
| **Document Path** | `reports/system-architecture/ui/04_user_bubble_component.md` |
| **Domain** | UI Architecture & Visual Card Presentation |
| **Target Persona** | Junior Android Developer (`CardView`, `MaterialCardView`, `shape.xml`, `LinearGradient`) |
| **Android Architectural Anchor** | `res/layout/item_chat_user.xml` + `res/drawable/bg_user_bubble.xml` + `CardView` |
| **Verification Mode** | Codebase Verified (`ui/chat_bubbles/user-bubble.slint`) |
| **Confidence** | VERIFIED |

> **TL;DR:** `ui/chat_bubbles/user-bubble.slint` renders right-aligned user prompt cards. It applies a modern Indigo-Violet linear gradient (`@linear-gradient(135deg)`), subtle glow borders, and a fixed 340px width constraint to prevent short-word geometry collapse.

---

## 🎨 1. Visual Card Architecture

In Android Studio, when styling a user chat bubble, you typically create an XML drawable (`res/drawable/bg_user_bubble.xml`) with a `<gradient>` tag and wrap your layout in a `MaterialCardView`.

In Slint, visual styling is applied directly to a `Rectangle` component using native declarative graphics properties:

```text
+-----------------------------------------------------------------------------+
|                          UserBubble VISUAL GEOMETRY                         |
+-----------------------------------------------------------------------------+
|  width: 340px, border-radius: 12px, clip: true                              |
|  background: @linear-gradient(135deg, #4338ca 0%, #312e81 100%)             |
|  border-color: #6366f188, drop-shadow-blur: 14px                            |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | Top Row: Sender Badge & Tag (HorizontalLayout, space-between)         |  |
|  | You                                                            PROMPT |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | Middle: Prompt Text (wrap: TextWrap.word-wrap)                        |  |
|  | "Analyze the latency of the last LangGraph turn and explain tool #2"  |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
|  +-----------------------------------------------------------------------+  |
|  | Bottom: Right-Aligned Timestamp (HorizontalLayout, alignment: end)     |  |
|  |                                                              10:45 AM |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+
```

### The Android ↔ Slint Mental Bridge

| Concept | Android Studio Equivalent | Slint (`user-bubble.slint`) Equivalent | Architectural Purpose |
| :--- | :--- | :--- | :--- |
| **Card Container** | `androidx.cardview.widget.CardView` | `export component UserBubble inherits Rectangle` | Provides rounded clipping, background fill, and drop shadow. |
| **Gradient Fill** | `<gradient android:angle="135" android:startColor="#4338ca" android:endColor="#312e81" />` | `@linear-gradient(135deg, #4338ca 0%, #312e81 100%)` | Creates modern, non-flat visual depth. |
| **Corner Radius** | `app:cardCornerRadius="12dp"` | `border-radius: 12px;` | Rounds all four corners smoothly. |
| **Elevation / Shadow** | `app:cardElevation="6dp"` | `drop-shadow-blur: 14px; drop-shadow-offset-y: 4px;` | Renders GPU-accelerated diffuse ambient shadow. |
| **Corner Clipping** | `android:clipToOutline="true"` | `clip: true;` | Ensures inner content does not bleed past rounded corners. |
| **Word Wrapping** | `android:inputType="textMultiLine"` | `wrap: TextWrap.word-wrap;` | Breaks long sentences across multiple lines cleanly. |

---

## 📐 2. The Fixed 340px Width Law (Anti-Collapse Discovery)

Notice line 15 of `ui/chat_bubbles/user-bubble.slint`:

```slint
export component UserBubble inherits Rectangle {
    in property <ChatMessage> msg;

    width: 340px;
    // ...
```

Why is `width` set to a fixed `340px` instead of dynamic wrap-content width?

### The "Short-Word Pillar" Bug

During early development of this desktop application, when a user typed a short message like `"hi"` or `"ok"`, a pure dynamic width calculation caused the card to shrink to the width of the letters `"hi"` (~18 pixels). Because the top row also contained the sender label `"You"` and the tag `"PROMPT"`, the layout engine squished the text horizontally and forced words to break into a vertical, 10-pixel-wide pillar:

```text
  DYNAMIC WIDTH FAILURE ("hi"):              FIXED 340px WIDTH (STABLE):
  +----+                                    +----------------------------------+
  | Y  |                                    | You                       PROMPT |
  | o  |                                    |                                  |
  | u  |                                    | hi                               |
  |    |                                    |                         10:45 AM |
  | h  |                                    +----------------------------------+
  | i  |
  +----+
```

In mobile Android development, you rarely notice this because the screen is narrow (360–400dp) and you set `android:minWidth="80dp"`. On a desktop window (760px to 1400px wide), an unconstrained card looks tiny or collapses. Fixing the card width at **340px** guarantees:
1. Short prompts maintain a beautiful, readable proportions.
2. Long prompts wrap cleanly with ample horizontal reading space.
3. The visual rhythm of the chat stream remains predictable and stable.

---

## 🔬 3. Code Walkthrough

```slint
import { ChatMessage } from "../types.slint";

export component UserBubble inherits Rectangle {
    in property <ChatMessage> msg;

    width: 340px;
    border-radius: 12px;
    border-width: 1px;
    border-color: #6366f188;
    background: @linear-gradient(135deg, #4338ca 0%, #312e81 100%);
    drop-shadow-blur: 14px;
    drop-shadow-color: #00000077;
    drop-shadow-offset-y: 4px;
    clip: true;

    VerticalLayout {
        padding-left: 14px;
        padding-right: 14px;
        padding-top: 10px;
        padding-bottom: 10px;
        spacing: 6px;

        // Top Row: Sender Pill Badge
        HorizontalLayout {
            alignment: space-between;
            Text {
                text: root.msg.sender;
                color: #c7d2fe;
                font-size: 11px;
                font-weight: 700;
                letter-spacing: 0.4px;
            }

            Text {
                text: "PROMPT";
                color: #818cf8;
                font-size: 9px;
                font-weight: 700;
            }
        }

        // Middle: User Message Text
        Text {
            text: root.msg.text;
            color: #f8fafc;
            font-size: 13.5px;
            wrap: TextWrap.word-wrap;
            horizontal-alignment: left;
        }

        // Bottom: Right-aligned Timestamp
        HorizontalLayout {
            alignment: end;
            Text {
                text: root.msg.timestamp;
                color: #a5b4fc;
                font-size: 10px;
            }
        }
    }
}
```

### Breakdown of Styling Details

- **`@linear-gradient(135deg, #4338ca 0%, #312e81 100%)`:** Renders a 135-degree diagonal gradient from vibrant Indigo (`#4338ca`) in the top-left corner to deep Night Violet (`#312e81`) in the bottom-right corner.
- **`border-color: #6366f188`:** The `88` suffix represents 50% alpha (transparency) in hex, creating a subtle neon-glass rim that catches light without being harsh.
- **`wrap: TextWrap.word-wrap`:** Ensures that if a user pastes a 200-word prompt, the text wraps at word boundaries rather than truncating with `...` or stretching the window horizontally.

---

## ⚡ 4. Zero-Telemetry Principle

Notice what `UserBubble` does **not** contain:
- No status dots.
- No RPM counters.
- No `inspect-ticket` callbacks.
- No live timers.

Because user messages are immutable once typed and sent, `UserBubble` consumes zero background CPU cycles. It is rendered directly by the GPU as a static cached geometry card, ensuring that even chat sessions with hundreds of past turns scroll smoothly at 144 Hz (6,944,444 ns frame budget).

---

## ⚠️ 5. Common Traps & Pitfalls for Android Developers

### Trap 1: Forgetting `clip: true` on Rounded Cards
- **The Mistake:** Defining `border-radius: 12px;` but omitting `clip: true;`.
- **The Reality:** In Slint, if child content (such as an inner layout or background fill) extends to the card edge, it will paint over the rounded corners unless clipping is enabled.
- **The Fix:** Always specify `clip: true;` on any card that uses `border-radius`.

### Trap 2: Using Pixel Dimensions for Typography
- **The Mistake:** Using `font-size: 14sp` from Android.
- **The Reality:** Slint uses logical pixels (`px` or `pt`). Writing `14sp` is a syntax error.
- **The Fix:** Write `font-size: 13.5px;`.

---

## ❓ 6. Q&A Log

| # | Question | Answer (concrete) | Date |
|---|---|---|---|
| Q1 | Why is `UserBubble` right-aligned if it doesn't specify `x` coordinates? | Alignment is controlled by the parent `HorizontalLayout` in `app-window.slint`, which specifies `alignment: msg.is-user ? end : start;`. `end` pushes the card to the far right. | 2026-11-04 |
| Q2 | Can the user select and copy text from `UserBubble`? | In Slint 1.16, static `Text` elements do not support mouse selection by default. To enable selection, `Text` can be swapped with a read-only `TextInput`. | 2026-11-04 |
| Q3 | What happens if `msg.timestamp` is empty? | The bottom `HorizontalLayout` renders an empty text element taking up 0 height, cleanly preserving layout geometry without errors. | 2026-11-04 |
| Q4 | Where is UserBubble located in the project tree? | It is located at `ui/chat_bubbles/user-bubble.slint` and included in the declarative module tree by `ui/chat_bubbles/ai-bubble.slint` and `ui/app-window.slint`. | 2026-11-15 |
