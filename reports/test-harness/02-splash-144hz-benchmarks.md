# 144 Hz Kyoto Dusk ASCII Splash Benchmarks & Verification

## Domain & Dependencies

| Attribute | Specification |
| :--- | :--- |
| **System** | `test-harness` (`tests/test_splash_loader.rs`, `tests/test_splash_on_tui.rs`) |
| **Domains Spanned** | `desktop` (`ai-workflow-gui` Rust rasterizer) · `assets` (`assests/coldwind_splash_144hz.json.zst`) |
| **Source of Truth Targets** | `../../src/splash_screen.rs`, `../../tests/test_splash_loader.rs`, `../../tests/test_splash_on_tui.rs` |
| **Governing Contracts** | `../../AGENTS.md` (Constitutional Iron Gates & Domain Separation) · `../system-architecture/07-ascii-splash-screen-engine.md` |
| **Verification Mode** | High-precision microbenchmarks (`std::time::Instant`, `cargo test --test test_splash_loader -- --nocapture`) |
| **Document Status** | `VERIFIED` |

---

## Executive Summary

> **TL;DR:** The Kyoto Dusk splash screen animation consists of 1,152 sequential ASCII art frames rendered at 144 Hz ($6.94\text{ ms}$ budget per frame). Microbenchmarks executed across three screen geometries reveal that $1280 \times 720$ (720p) rasterizes in **$5.07\text{ ms}$**, comfortably locking the 144 FPS target with $1.87\text{ ms}$ of headroom. At $1920 \times 1080$ ($10.52\text{ ms}$) and $2560 \times 1440$ ($18.61\text{ ms}$), CPU software blitting exceeds the 144 Hz frame budget, validating the design decision to set the desktop window's default geometry to $1280 \times 720$ (16:9).

---

## 1. Asset & Animation Dimensions

| Dimension | Value | Architectural Notes |
| :--- | :--- | :--- |
| **Total Frames** | 1,152 frames | 8.0 seconds total playback at 144 Hz ($1152 / 144 = 8\text{ s}$). |
| **Grid Resolution** | $160 \times 64$ cells | 10,240 cells per frame, each carrying a `char` and 24-bit RGB tuple. |
| **Uncompressed JSON** | ~60 MB | Full text representation in `assests/coldwind_splash_144hz.json`. |
| **Compressed Zstandard** | ~953 KB | Compressed at `zstd -19` into `assests/coldwind_splash_144hz.json.zst`. |
| **Embedded `.rodata`** | ~1.43 MB | Embedded directly into the Rust binary via `include_bytes!`. |
| **Font Asset** | JetBrains Mono Nerd Font | `JetBrainsMonoNLNerdFont-Regular.ttf` embedded for glyph rasterization. |

---

## 2. Multi-Resolution Rasterization Benchmark Results

Measurements captured using `std::time::Instant` inside `tests/test_splash_loader.rs` on a standard Linux 144 Hz display workstation:

| Screen Resolution | Aspect Ratio | Cell Box ($W \times H$) | Font Size | Raster Time | 144 Hz Budget ($6.94\text{ ms}$) | Verdict |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **$1280 \times 720$ (720p)** | 16:9 | $8 \times 11\text{ px}$ | 11.0 pt | **$5.07\text{ ms}$** | $6.94\text{ ms}$ ($+1.87\text{ ms}$ headroom) | **PASS — 144 FPS Locked** |
| **$1920 \times 1080$ (1080p)** | 16:9 | $12 \times 17\text{ px}$ | 16.5 pt | **$10.52\text{ ms}$** | $6.94\text{ ms}$ ($-3.58\text{ ms}$ over) | **FAIL — Drops to ~95 FPS** |
| **$2560 \times 1440$ (1440p)** | 16:9 | $16 \times 22\text{ px}$ | 22.0 pt | **$18.61\text{ ms}$** | $6.94\text{ ms}$ ($-11.67\text{ ms}$ over) | **FAIL — Drops to ~54 FPS** |

```text
Frame Budget Comparison (144 Hz Target: 6.94 ms)
┌────────────────────────────────────────────────────────┐
│ 1280x720:  [■■■■■■■■■■■■■■■] 5.07 ms (PASS)            │
│ Target:    [■■■■■■■■■■■■■■■■■■■■] 6.94 ms              │
│ 1920x1080: [■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■] 10.52 ms   │
│ 2560x1440: [■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■] │
│            18.61 ms                                    │
└────────────────────────────────────────────────────────┘
```

---

## 3. Glyph Caching & Memory Profiling

### Warm-Up Glyph Generation
Rather than rasterizing character glyphs during the active animation loop:
- The first 10 frames are scanned at boot to discover all distinct printable characters.
- Exactly **38 unique ASCII characters** are identified.
- `Fontdue` generates 8-bit alpha coverage bitmaps for each character once.
- Warm-up duration: **$61.60\text{ ms}$** (one-time cost incurred during `AppState::Booting`).
- Warm-up RAM footprint: < 50 KB for all 38 cached glyph bitmaps.

```text
Glyph Cache Initialization Pipeline:
┌────────────────────────┐      ┌────────────────────────┐      ┌────────────────────────┐
│ Scan First 10 Frames   ├─────►│ Identify 38 Characters ├─────►│ Fontdue Rasterization   │
│ (102,400 raw cells)    │      │ (Unique ASCII Set)     │      │ (8-bit alpha bitmaps)  │
└────────────────────────┘      └────────────────────────┘      └───────────┬────────────┘
                                                                            │
                                                                            ▼
                                                                ┌────────────────────────┐
                                                                │ Store in HashMap Cache │
                                                                │ (< 50 KB in RAM)       │
                                                                └────────────────────────┘
```

### Memory Allocation & Latency Profile

| Pipeline Stage | Allocation Target | Size / Buffer Footprint | Alloc Frequency | Reclaim / Lifetime |
| :--- | :--- | :--- | :--- | :--- |
| **Asset Load** | Decompressed JSON slice | ~60 MB heap | Once at cold boot | Dropped after `serde_json` parse |
| **Frame Array** | `Vec<Frame>` (1,152 items) | ~38 MB heap | Once at cold boot | Retained in memory until `AppState::Ready` |
| **Glyph Bitmaps** | `HashMap<char, GlyphData>` | ~48 KB heap | Once at cold boot | Retained across animation loop |
| **Pixel Target** | RGBA Frame Buffer | $1280 \times 720 \times 4 \approx 3.68\text{ MB}$ | Allocated once | Mutated in-place every 6.94 ms tick |
| **Slint Image** | `slint::Image::from_rgba8` | Wrapped pixel slice | Every frame tick | Consumed by GPU texture upload |

---

## 4. Terminal TUI Preview Architecture (`test_splash_on_tui.rs`)

`tests/test_splash_on_tui.rs` validates ANSI color fidelity without needing a running Wayland or X11 compositor:
- **24-Bit TrueColor Sequences:** Each character is emitted with `\x1B[38;2;R;G;Bm<char>`.
- **System Call Batching:** Standard output is locked via `io::stdout().lock()` and wrapped in an 8 KB `BufWriter`. This reduces 10,240 individual system calls per frame to a single batched flush.
- **In-Place Redraw:** The terminal cursor resets to row 1, column 1 using `\x1B[1;1H`, eliminating terminal screen flickering.
- **Pacing Control:** Playback paces at ~30 FPS (33 ms per frame) for terminal inspection comfort.

---

## 5. Code Hygiene & Optimization Opportunities

The benchmark audit identifies two optimization opportunities for future sprints:

1. **Static Gamma LUT:**
   Currently, `draw_pixels_on_buffer` constructs a gamma lookup table dynamically. Replacing this with a static `const [u16; 256]` array generated via `powf(0.7)` eliminates repeated setup overhead.
2. **Embedded Asset Usage in Tests:**
   `tests/test_splash_loader.rs` currently reads the raw 60 MB JSON file from disk (`assests/coldwind_splash_144hz.json`). Migrating tests to use `load_embedded_json_frames()` from `src/splash_screen.rs` tests the true production Zstandard decompression path and speeds up test suite execution.

---

## Q&A Log

| # | Question | Answer (concrete) | Date |
| :--- | :--- | :--- | :--- |
| Q1 | Why does $1280 \times 720$ hit 144 FPS while $1920 \times 1080$ misses it? | Software CPU rasterization must blit $160 \times 64 = 10,240$ character glyphs per frame. At $1280 \times 720$, each glyph occupies $8 \times 11 = 88$ pixels ($901,120$ pixel writes). At $1920 \times 1080$, each glyph occupies $12 \times 17 = 204$ pixels ($2,088,960$ pixel writes), doubling the CPU memory bandwidth required. | 2026-11-15 |
| Q2 | Why is 1,152 frames significant? | At 144 frames per second, 1,152 frames provides exactly $1152 / 144 = 8.0$ seconds of animation. Older test drafts asserted 864 frames (which corresponded to 60 FPS for 14.4 seconds), causing test assertion failures. | 2026-11-15 |
| Q3 | Why does the engine use Zstandard (`zstd -19`) compression? | The raw JSON animation file is ~60 MB. Zstandard compression at level 19 reduces the file to ~953 KB, allowing it to be embedded directly into the compiled executable's `.rodata` section via `include_bytes!` without ballooning binary size. | 2026-11-15 |
| Q4 | How does `test_splash_on_tui` avoid terminal flicker? | It wraps stdout in a `BufWriter` and resets cursor position to `\x1B[1;1H` instead of clearing the screen on every frame tick. | 2026-11-15 |
