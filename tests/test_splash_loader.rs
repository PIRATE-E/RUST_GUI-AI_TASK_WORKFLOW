//! Smoke test validating splash screen asset loading and glyph cache generation.

use std::path::PathBuf;
use std::time::Instant;

slint::include_modules!();

use ai_workflow_gui::splash_screen::{build_glyph_cache, load_json_frame};

#[test]
fn test_splash_asset_parsing() {
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assests/coldwind_splash_144hz.json");

    assert!(json_path.exists(), "Asset file must exist at {json_path:?}");

    let start = Instant::now();
    let frames = load_json_frame(&json_path).expect("load_json_frame must parse successfully");
    let load_time = start.elapsed();

    println!(
        "\n[test_splash_loader] ✅ Parsed {} frames in {:.2?}",
        frames.len(),
        load_time
    );

    assert_eq!(frames.len(), 1152, "Total frame count must be 1152");

    let start_cache = Instant::now();
    let cache = build_glyph_cache(&frames[..10], 22.0);
    let cache_time = start_cache.elapsed();

    println!(
        "[test_splash_loader] ✅ Cached {} unique glyphs in {:.2?}",
        cache.len(),
        cache_time
    );
    assert!(!cache.is_empty(), "Glyph cache must not be empty");

    // Benchmark draw_pixels_on_buffer at 2560x1440
    use ai_workflow_gui::splash_screen::{draw_pixels_on_buffer, ScreenWindow};
    let screen_2560 = ScreenWindow { width: 2560, height: 1440 };
    let mut buf_2560 = vec![0u8; 2560 * 1440 * 4];
    let t0 = Instant::now();
    let _img = draw_pixels_on_buffer(&screen_2560, &mut buf_2560, &frames[0], &cache, 16, 22);
    let render_time_2560 = t0.elapsed();
    println!("[test_splash_loader] ⏱️ 2560x1440 frame render time: {:.2?}", render_time_2560);

    // Benchmark draw_pixels_on_buffer at 1920x1080
    let cache_1920 = build_glyph_cache(&frames[..10], 16.5);
    let screen_1920 = ScreenWindow { width: 1920, height: 1080 };
    let mut buf_1920 = vec![0u8; 1920 * 1080 * 4];
    let t1 = Instant::now();
    let _img = draw_pixels_on_buffer(&screen_1920, &mut buf_1920, &frames[0], &cache_1920, 12, 17);
    let render_time_1920 = t1.elapsed();
    println!("[test_splash_loader] ⏱️ 1920x1080 frame render time: {:.2?}", render_time_1920);

    // Benchmark draw_pixels_on_buffer at 1280x720
    let cache_1280 = build_glyph_cache(&frames[..10], 11.0);
    let screen_1280 = ScreenWindow { width: 1280, height: 720 };
    let mut buf_1280 = vec![0u8; 1280 * 720 * 4];
    let t2 = Instant::now();
    let _img = draw_pixels_on_buffer(&screen_1280, &mut buf_1280, &frames[0], &cache_1280, 8, 11);
    let render_time_1280 = t2.elapsed();
    println!("[test_splash_loader] ⏱️ 1280x720 frame render time: {:.2?}", render_time_1280);
}
