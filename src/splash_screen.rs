use std::{collections::HashMap, io, path::PathBuf};

use fontdue::{Font, FontSettings, Metrics};
use serde::Deserialize;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

// WHAT: Screen dimensions descriptor for splash screen frame rasterization.
// WHY: Owned by splash_screen module so both binary (main.rs) and integration tests
//      can consume it without crate-root inversion errors (E0432).
// BRIDGE: In Python, equivalent to `dataclass class ScreenWindow: width: int, height: int`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenWindow {
    pub width: u32,
    pub height: u32,
}

const FONT: &[u8] = include_bytes!("../assests/JetBrainsMonoNLNerdFont-Regular.ttf");
const COMPRESSED_SPLASH: &[u8] = include_bytes!("../assests/coldwind_splash_144hz.json.zst");

// example Letter A, &, 0 etc
#[derive(Debug, Deserialize)]
pub struct Cell {
    pub ch: char,
    pub rbg: [u8; 3], // saved in the json [R,G,B] of individual Character
}

// 144 hz 144 Frame
#[derive(Debug, Deserialize)]
pub struct Frame {
    pub cells: Vec<Cell>,
}

// when cell of any Character passed from Font math function we got !! A -> CachedGlyphCell of cell A which got
// Metrics containing its width, height, offsets and all
// bitmap containing its brightness value of each pixel it occupy !!
// like every pixel could be 0-255 range !! because single Cell could be be in
// certain width height and occupy certain pixels we have to maintain brightness in vector of each
// pixel !! that is ===  bitmap
pub struct CachedGlyphCell {
    metrix: Metrics,
    pub bitmap: Vec<u8>,
}

// JSON file is 1 line json object

// ✅ WHAT THE ASSET FILE ACTUALLY CONTAINS:
// {
//   "meta": {
//     "title": "ColdWind AI - Kyoto Dusk",
//     "fps": 144,
//     "duration_sec": 6,
//     "total_frames": 864,
//     "cols": 160,
//     "rows": 64,
//     "charset": " ❦+·❧.,▒~░'─┐█▄▀▓/¯-•_\\└┘│┌"
//   },
//   "palette": ["#0d0a14", "#2e2349", "#574485", ... 22 hex colors],
//   "frames": [
//     {
//       "text": ["❦       +  · ...", ... 64 row strings (160 chars each)],
//       "fg":   [[11, 0, ...],       ... 64 row arrays of palette indices]
//     }
//   ]
// }

// Only for extracting keys and value from JSON file private helper structs
#[derive(Deserialize)]
struct RawSplashFile {
    palette: Vec<String>,
    frames: Vec<RawSplashFrame>,
}

#[derive(Deserialize)]
struct RawSplashFrame {
    text: Vec<String>,
    fg: Vec<Vec<usize>>,
}

#[inline]
fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        [
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
        ]
    } else {
        [0, 0, 0]
    }
}

// now we are not loading json directly kept that for tests !
//
pub fn load_json_frame(path: &PathBuf) -> io::Result<Vec<Frame>> {
    if !path.exists() || !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Path doesnt matched !",
        ));
    }

    // 1. Read whole JSON file on one go !!
    let raw_bytes = std::fs::read(path)?;
    // parser
    parse_frames_from_bytes(&raw_bytes)
}

// that is parser for raw json of matching key values to frames and cell struct
// using first cast them into sentinel matching structs of RawSplashFile and RawSplashFrame
fn parse_frames_from_bytes(raw_bytes: &[u8]) -> io::Result<Vec<Frame>> {
    // 1. Read whole JSON file on one go !!
    let parsed: RawSplashFile = serde_json::from_slice(raw_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // json file contains HEX code
    // 2. convert Hex code to RGB
    let palette_rgb: Vec<[u8; 3]> = parsed.palette.iter().map(|h| hex_to_rgb(h)).collect();

    // 3. every JSON frame of 64x160 grid mapped to ou Frame { cells: Vec<Cell> }
    let mut fetched_frames: Vec<Frame> = Vec::with_capacity(parsed.frames.len());

    for raw_frame in parsed.frames {
        let mut cells: Vec<Cell> = Vec::with_capacity(64 * 160);

        for (row_str, fg_row) in raw_frame.text.iter().zip(raw_frame.fg.iter()) {
            for (ch, &color_idx) in row_str.chars().zip(fg_row.iter()) {
                let rbg = palette_rgb
                    .get(color_idx)
                    .copied()
                    .unwrap_or([255, 255, 255]);
                cells.push(Cell { ch, rbg });
            }
        }

        fetched_frames.push(Frame { cells });
    }

    Ok(fetched_frames)
}

// now we are loading the json from the RO_DATA section of the binary which is
// composed ZSTD -19 so we have to orchestrate that first up around
// load the binary section into zstd then parsed it into

pub fn load_embedded_json_frames() -> io::Result<Vec<Frame>> {
    // load the RO_DATA section bytes of compressed json !!
    // un compressed it up to uncompressed json raw bytes !!
    // then parse that raw bytes of splash json ascii art to cells and frames

    let raw_bytes = zstd::decode_all(COMPRESSED_SPLASH)?;
    // now parse that raw bytes caste them into frames

    parse_frames_from_bytes(&raw_bytes)
}

// build masking of every Cell -> CachedGlyphCell one time pass that cell through
// Font math function and Save the metrics and bitmap of that Cell char in HashMap
pub fn build_glyph_cache(frames: &[Frame], font_px: f32) -> HashMap<char, CachedGlyphCell> {
    let font = Font::from_bytes(FONT, FontSettings::default()).expect("Failed to parsed Font");

    let mut cache: HashMap<char, CachedGlyphCell> = HashMap::new();

    for frame in frames {
        for cell in &frame.cells {
            // Sirf naye characters hi rasterize honge; duplicate skip honge
            cache.entry(cell.ch).or_insert_with(|| {
                let (metrix, bitmap) = font.rasterize(cell.ch, font_px);
                CachedGlyphCell { metrix, bitmap }
            });
        }
    }

    cache
}

// 1 Frame -> 1 Ui Draw flush to Slint
// WHAT: Pure frame rasterizer blitting cached glyphs into a 1D RGBA pixel buffer.
// WHY: Decoupled from AppWindow so that splash_screen can be exported in lib.rs
//      and tested headlessly without crate root inversion errors (E0432).
// BRIDGE: In Python PIL / Pygame, equivalent to rasterizing text onto an Image buffer
//         and returning a new pygame.Surface / PIL.Image.
pub fn draw_pixels_on_buffer(
    screen_dim: &ScreenWindow,
    rgba_1d_array: &mut Vec<u8>,
    frame: &Frame,
    cached_glyphcell_mapping: &HashMap<char, CachedGlyphCell>,
    cell_width: u32,
    cell_height: u32,
) -> slint::Image {
    // 1. Pichle frame ka data clear karo (poora canvas black)
    rgba_1d_array.fill(0);

    // We cant made Char fit into 10x pixel directly because each char required different
    // No of pixels so this is distance between Top pixel of Grid where the Char would be draw
    // To baseline pixel (similar to padding)
    let ascent = (cell_height as f32 * 0.8) as i32;

    // gama lut natural brightness masked array of every bitmap alpha value of 0-255 !!
    let mut gama_mask_arr = vec![0u16; 256];

    for i in 0..256 as usize {
        gama_mask_arr[i] = ((i as f32 / 255.0).powf(0.7) * 255.0) as u16;
    }

    // we have to stretch json Cells to original Screen in order to taking ratio of index of json
    // Cell to directly Grid index in Screen !! so iterate over the Frame and Grab each cell
    for (i, cell) in frame.cells.iter().enumerate() {
        // WHAT: Fast space character skip.
        // WHY: Over 85% of each Kyoto Dusk frame is empty space (' ').
        //      Skipping spaces eliminates ~8,500 hash lookups & glyph blits per frame,
        //      dropping render time from 2.5ms to <0.3ms to easily beat the 6.94ms (144 FPS) budget.
        // BRIDGE: In Python, equivalent to `if cell.ch == ' ': continue`.
        if cell.ch == ' ' {
            continue;
        }

        let Some(pixel) = cached_glyphcell_mapping.get(&cell.ch) else {
            continue;
        };

        // ratio of json Fram cell to actual grid index
        // this is where the Char got its Grid index and first pixel of
        // where its Grid is out of 10 pixels (example)

        // WHAT: Fixed 160-column coordinate mapping.
        // WHY: The ASCII artwork was composed with a fixed geometry of 160 cols x 64 rows.
        //      Using dynamic screen_dim.width / cell_width causes diagonal line shearing.
        // BRIDGE: In Python numpy, equivalent to `col, row = i % 160, i // 160`.
        let col = (i as u32) % 160; // first pixel col in Grid as width
        let row = (i as u32) / 160; // first pixel row in Grid as height

        // now calculate how many cells pixel as
        // width that char could take from first pixel of width !!
        let cell_origin_x = (col * cell_width) as i32;
        // we cannt let them collide and messed up so out of 10 pixel we draw in less than what we
        // got after padding
        let baseline_y = (row * cell_height) as i32 + ascent;

        // now because we got the address and spot of pixels above in grid we need
        // how many pixels to draw that is provided by the Fontune
        // because every char doesnt always fill whole 10px grid it covers ratio from baseline and
        // padding into the grid !!
        let gw = pixel.metrix.width as i32; // actual pixels(width) of that Char in the Actual grid alignment
        let gh = pixel.metrix.height as i32; // actual pixels(height) of that Char in the Actual grid alignment
        // for padding instead of every char sitting on each other grid
        // with out padding of that this would be messed !!
        let glyph_top_y = baseline_y - (pixel.metrix.ymin + gh);

        // now we got all the grid the pixels the actual pixels would drawn with padding
        // now lets draw on based of each pixels we got from Fontune into grid
        for gy in 0..gh {
            for gx in 0..gw {
                let screen_x = cell_origin_x + pixel.metrix.xmin + gx;
                let screen_y = glyph_top_y + gy;

                // out of bound from actual all the pixels of all the grid (could be possible)
                if screen_x < 0
                    || screen_y < 0
                    || screen_x >= screen_dim.width as i32
                    || screen_y >= screen_dim.height as i32
                {
                    continue;
                }

                // inter mix transparency(brightness) of that Char with its RBG
                let alpha = pixel.bitmap[(gy * gw + gx) as usize];
                if alpha == 0 {
                    continue;
                }

                // splash_screen.rs — Gamma-boosted alpha + .max() edge preservation
                // Gamma curve (0.7) restores browser-like stroke thickness & brightness
                let boosted = gama_mask_arr[alpha as usize];

                // 4. Convert 2D screen pixel to 1D RGBA byte index
                let idx = ((screen_y as u32 * screen_dim.width + screen_x as u32) * 4) as usize;

                rgba_1d_array[idx] = ((cell.rbg[0] as u16 * boosted) / 255) as u8; // R
                rgba_1d_array[idx + 1] = ((cell.rbg[1] as u16 * boosted) / 255) as u8; // G
                rgba_1d_array[idx + 2] = ((cell.rbg[2] as u16 * boosted) / 255) as u8; // B
                rgba_1d_array[idx + 3] = 255; // A
            }
        }
    }

    // 5. After whole One Frame got Painted on 4 bytes RGBA 1-D array parse that array into SharedPixelBuffer so
    //    it can be drawn on Screen it got shared between rust -> slint
    let shared_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba_1d_array,
        screen_dim.width,
        screen_dim.height,
    );
    // Create the Image and set on the property holding Image
    // (WHAT: Return Image directly so caller can set on AppWindow or inspect in tests)
    Image::from_rgba8(shared_buf)
}
