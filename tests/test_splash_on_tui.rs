use ai_workflow_gui::splash_screen::*;
use std::{
    io::{self, BufWriter, Write},
    path::PathBuf,
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[inline]
fn clear_screen() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").args(["/C", "cls"]).status();
    } else {
        let _ = Command::new("clear").status();
    }
}

#[test]
fn test_splash_screen_on_tui() -> io::Result<()> {
    // Resolve the full path to the JSON asset relative to Cargo.toml
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assests/coldwind_splash_144hz.json");

    assert!(json_path.exists(), "Asset file must exist at {json_path:?}");

    // Load all flattened frames using the library parser
    let frames: Vec<Frame> = load_json_frame(&json_path)?;
    assert!(!frames.is_empty(), "Frames vector must not be empty");

    println!("Loaded {} frames. Starting TUI preview...", frames.len());
    thread::sleep(Duration::from_secs(2));

    // Clear the terminal screen once using the native OS command
    clear_screen();

    // Lock stdout and wrap it in a BufWriter to prevent per-character syscall overhead
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    // Each row in the JSON matrix consists of 160 characters
    const GRID_COLS: usize = 160;

    for frame in &frames {
        let frame_start = Instant::now();

        // Move the terminal cursor to top-left (row 1, col 1) for flicker-free overwriting
        write!(out, "\x1B[1;1H")?;

        // Slice the flat 1D `cells` vector into 160-character rows and print with ANSI RGB colors
        for row_cells in frame.cells.chunks(GRID_COLS) {
            for cell in row_cells {
                // Apply 24-bit ANSI truecolor foreground (\x1B[38;2;R;G;Bm) for each cell
                write!(
                    out,
                    "\x1B[38;2;{};{};{}m{}",
                    cell.rbg[0], cell.rbg[1], cell.rbg[2], cell.ch
                )?;
            }
            // Reset color at the end of each line and move to the next row
            writeln!(out, "\x1B[0m")?;
        }

        // Flush the entire frame buffer to the terminal at once
        out.flush()?;

        // Pace playback to ~30 FPS (~33ms per frame) for smooth terminal inspection
        let elapsed = frame_start.elapsed();
        if elapsed < Duration::from_millis(33) {
            thread::sleep(Duration::from_millis(33) - elapsed);
        }
    }

    // Reset terminal colors and clear the screen after animation finishes
    write!(out, "\x1B[0m")?;
    out.flush()?;
    clear_screen();

    println!("TUI preview finished ({} frames).", frames.len());
    Ok(())
}
